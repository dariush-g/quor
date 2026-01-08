use std::collections::{HashMap, HashSet};

use crate::{ir::block::*, lexer::ast::*};

#[derive(Default, Debug, Clone)]
pub struct BlockIdGen {
    pub next: usize,
}

impl BlockIdGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn fresh(&mut self) -> BlockId {
        let id = BlockId(self.next);
        self.next += 1;
        id
    }
}

#[derive(Default)]
pub struct VRegGenerator {
    pub next: usize,
}

impl VRegGenerator {
    pub fn fresh(&mut self) -> VReg {
        let reg = VReg(self.next);
        self.next += 1;
        reg
    }
}

#[derive(Debug, Default, Clone)]
pub struct VarGenerator {
    pub next: usize,
}

impl VarGenerator {
    pub fn fresh(&mut self) -> usize {
        let id = self.next;
        self.next += 1;
        id
    }
}

#[derive(Default)]
pub struct IRGenerator {
    pub vreg_gen: VRegGenerator,
    pub block_gen: BlockIdGen,
    pub var_gen: VarGenerator,
    pub var_map: HashMap<String, (Type, usize)>,
    pub blocks: Vec<IRBlock>,
    pub globals: HashMap<String, GlobalDef>,
    pub ir_program: IRProgram,
    pub block_scope_handler: BlockScopeHandler,
}

#[derive(Clone, Debug, Default)]
pub struct BlockScopeHandler {
    pub closed: HashSet<BlockId>,
    pub open: BlockId,
}

impl IRGenerator {
    pub fn new_scope(&mut self) -> BlockId {
        let id = self.block_gen.fresh();
        self.block_scope_handler
            .closed
            .insert(self.block_scope_handler.open);
        self.block_scope_handler.open = id;
        id
    }

    pub fn add_new_var(&mut self, name: String, ty: Type) -> Value {
        let id = self.var_gen.fresh();
        self.var_map.insert(name, (ty, id));
        Value::Local(id)
    }

    pub fn generate(mut stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_generator = IRGenerator::default();
        for (i, stmt) in stmts.clone().iter().enumerate() {
            if let Stmt::AtDecl(..) = stmt {
                ir_generator.generate_declaration(stmt)?;
                stmts.remove(i);
            };
        }
        for stmt in stmts {
            match stmt {
                Stmt::FunDecl { .. } => ir_generator.generate_function(&stmt),
                Stmt::StructDecl { .. } => ir_generator.generate_struct(&stmt),
                _ => Ok(()),
            }?
        }

        Err(String::from("error"))
    }

    fn generate_struct(&mut self, stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_declaration(&mut self, stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_function(&mut self, func: &Stmt) -> Result<(), String> {
        if let Stmt::FunDecl {
            name,
            params: parameters,
            return_type,
            body,
            attributes,
        } = func
        {
            let mut params = vec![];
            for _ in 0..parameters.len() {
                params.push(self.vreg_gen.fresh());
            }

            let ir_func = IRFunction {
                name: name.clone(),
                params,
                ret_type: return_type.clone(),
                blocks: todo!(),
                entry: todo!(),
                attributes: todo!(),
            };
        }

        Ok(())
    }

    fn parse_block(&mut self, stmts: Vec<Stmt>) -> BlockId {
        let scope = self.new_scope();
        let mut instructions = Vec::new();
        let mut terminator: Terminator = Terminator::Return { value: None };
        for stmt in stmts {
            match stmt {
                Stmt::AtDecl(dec, content, ..) => match dec.as_str() {
                    "__asm__" | "_asm_" | "asm" => {
                        instructions.push(IRInstruction::Declaration(AtDecl::InlineAssembly {
                            content: content.unwrap(),
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
                    let val = self.add_new_var(name, var_type);
                    self.emit_into_local(val, value, &mut instructions);
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
                } => {
                    let terminator = Terminator::Branch {
                        condition: todo!(),
                        if_true: todo!(),
                        if_false: todo!(),
                    };
                }
                Stmt::While { condition, body } => todo!(),
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => todo!(),
                Stmt::Block(stmts) => todo!(),
                Stmt::Expression(expr) => {
                    self.first_pass_parse_expr(expr, &mut instructions);
                }
                Stmt::Return(expr) => {
                    if let Some(expr) = expr {
                        if let Some((val, ty)) = self.first_pass_parse_expr(expr, &mut instructions)
                        {
                            terminator = Terminator::Return { value: Some(val) }
                        }
                    } else {
                        terminator = Terminator::Return { value: None }
                    }
                }
                Stmt::Break => todo!(),
                Stmt::Continue => todo!(),
            }
        }

        let block = IRBlock {
            id: scope,
            instructions,
            terminator,
        };

        self.blocks.push(block);

        scope
    }

    fn first_parse_full_block(&mut self, vec_stmt: Vec<Stmt>) -> Vec<IRBlock> {
        let mut blocks = Vec::new();
        let mut current: Vec<IRInstruction> = Vec::new();
        let ret = None;
        for stmt in vec_stmt {
            match stmt {
                Stmt::AtDecl(dec, content, ..) => match dec.as_str() {
                    "__asm__" | "_asm_" | "asm" => {
                        current.push(IRInstruction::Declaration(AtDecl::InlineAssembly {
                            content: content.unwrap(),
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
                    let val = self.add_new_var(name, var_type);
                    self.emit_into_local(val, value, &mut current);
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
                } => {
                    let terminator = Terminator::Branch {
                        condition: todo!(),
                        if_true: todo!(),
                        if_false: todo!(),
                    };
                }
                Stmt::While { condition, body } => todo!(),
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => todo!(),
                Stmt::Block(stmts) => todo!(),
                Stmt::Expression(expr) => todo!(),
                Stmt::Return(expr) => {
                    blocks.push(IRBlock {
                        id: self.block_gen.fresh(),
                        instructions: current,
                        terminator: Terminator::Return { value: ret.clone() },
                    });

                    current = Vec::new();
                }
                Stmt::Break => todo!(),
                Stmt::Continue => todo!(),
            }
        }

        blocks
    }
}
