use crate::{
    ir::{
        block::{AtDecl, BlockId, IRInstruction, Terminator},
        cfg::IRGenerator,
    },
    lexer::ast::{Expr, Stmt},
};

impl IRGenerator {
    pub fn set_terminator(&mut self, block: BlockId, terminator: Terminator) {
        if let Terminator::TemporaryNone = terminator {
            panic!("Cannot finalize block terminator as None :: BlockId: {block:?}");
        }

        self.blocks[block.0].terminator = terminator;
    }

    pub fn lower_while(&mut self, cond: Expr, body: &[Stmt]) {
        let cond_block = self.new_block();
        let body_block = self.new_block();
        let after_block = self.new_block();

        self.set_terminator(
            self.scope_handler.current,
            Terminator::Jump { block: cond_block },
        );
        self.set_current(cond_block);

        let (cond_value, _cond_type) = self
            .lower_place(cond)
            .expect("Error in fn lowering while :: getting cond_value :: lowering.rs");

        self.set_terminator(
            cond_block,
            Terminator::Branch {
                condition: cond_value,
                if_true: body_block,
                if_false: after_block,
            },
        );

        self.scope_handler.break_stack.push_front(after_block);
        self.scope_handler.continue_stack.push_front(cond_block);

        self.set_current(body_block);
        self.lower_block(body);

        self.scope_handler.break_stack.pop_front();
        self.scope_handler.continue_stack.pop_front();

        if let Terminator::TemporaryNone = self.blocks[self.scope_handler.current.0].terminator {
            self.set_terminator(
                self.scope_handler.current,
                Terminator::Jump { block: cond_block },
            );
        }
    }

    pub fn lower_if(&mut self, cond: Expr, if_true: &[Stmt], else_: Option<&[Stmt]>) {
        let cond_block = self.new_block();
        let if_true_block = self.new_block();
        let if_false_block = else_.map(|_else_| self.new_block());
        let continue_block = self.new_block();

        let (value, _) = self.lower_place(cond).unwrap();

        self.set_terminator(
            self.scope_handler.current,
            Terminator::Jump { block: cond_block },
        );

        self.set_current(cond_block);

        if let Some(if_false_block) = if_false_block {
            self.set_terminator(
                self.scope_handler.current,
                Terminator::Branch {
                    condition: value,
                    if_true: if_true_block,
                    if_false: if_false_block,
                },
            );
            self.set_terminator(
                if_false_block,
                Terminator::Jump {
                    block: continue_block,
                },
            );
        } else {
            self.set_terminator(
                self.scope_handler.current,
                Terminator::Branch {
                    condition: value,
                    if_true: if_true_block,
                    if_false: continue_block,
                },
            );
        }

        self.set_current(if_true_block);
        self.lower_block(if_true);

        if let Terminator::TemporaryNone = self.blocks[self.scope_handler.current.0].terminator {
            self.set_terminator(
                self.scope_handler.current,
                Terminator::Jump {
                    block: continue_block,
                },
            );
        }
    }

    pub fn lower_block(&mut self, body: &[Stmt]) {
        for stmt in body {
            match stmt {
                Stmt::AtDecl(dec, content, ..) => match dec.as_str() {
                    "__asm__" | "_asm_" | "asm" => {
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::Declaration(AtDecl::InlineAssembly {
                                content: content.clone().unwrap(),
                            }));
                    }
                    _ => {
                        eprintln!("warning :: unexpected declaration: \"{dec}\"");
                    }
                },
                Stmt::VarDecl {
                    name,
                    value,
                    var_type,
                } => {
                    let val = self.add_new_var(name.to_string(), var_type.clone());
                    self.emit_into_local(val, value.clone());
                }
                Stmt::FunDecl { name, .. } => {
                    eprintln!("warning :: function {name} defined inside a block")
                }
                Stmt::StructDecl { name, .. } => {
                    eprintln!("warning :: struct {name} defined a block")
                }
                Stmt::If {
                    condition,
                    then_stmt,
                    else_stmt,
                } => self.lower_if(
                    condition.clone(),
                    (**then_stmt).as_block(),
                    else_stmt.as_ref().map(|s| s.as_block()),
                ),
                Stmt::While { condition, body } => {
                    if let Stmt::Block(stmts) = *body.clone() {
                        self.lower_while(condition.clone(), &stmts);
                    }
                }
                Stmt::For { .. } => {}
                Stmt::Block(stmts) => {
                    self.lower_block(stmts);
                }
                Stmt::Expression(expr) => {
                    self.first_pass_parse_expr(expr.clone());
                }
                Stmt::Return(expr) => {
                    let mut value = None;
                    if let Some(expr) = expr {
                        value = Some(self.first_pass_parse_expr(expr.clone()).unwrap().0);
                    }
                    self.set_terminator(self.scope_handler.current, Terminator::Return { value });
                }
                Stmt::Break => {
                    let break_scope = self.scope_handler.break_stack.pop_front().unwrap();
                    self.set_terminator(
                        self.scope_handler.clone().current,
                        Terminator::Jump { block: break_scope },
                    );
                }
                Stmt::Continue => {
                    let continue_scope = self.scope_handler.continue_stack.pop_front().unwrap();
                    self.set_terminator(
                        self.scope_handler.clone().current,
                        Terminator::Jump {
                            block: continue_scope,
                        },
                    );
                }
            }
        }
    }
}
