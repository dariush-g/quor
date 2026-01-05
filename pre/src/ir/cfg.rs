use std::collections::HashMap;

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

#[derive(Default)]
pub struct IRGenerator {
    pub vreg_gen: VRegGenerator,
    pub block_gen: BlockIdGen,
    pub blocks: Vec<IRBlock>,
    pub globals: HashMap<String, GlobalDef>,
}

impl IRGenerator {
    fn generate(mut stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let program = IRProgram::default();
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
            params,
            return_type,
            body,
            attributes,
        } = func
        {
            let mut params = vec![];

            for _ in 0..params.len() {
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

    fn first_parse_full_block(&mut self, vec_stmt: Vec<Stmt>) -> Vec<IRBlock> {
        let mut blocks = Vec::new();
        let mut current: Vec<IRInstruction> = Vec::new();
        let mut var_map: HashMap<String, (Option<usize>, Option<VReg>)> = HashMap::new();
        let mut var_count = 0_usize;
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
                Stmt::VarDecl { name, value, .. } => {
                    let id = var_count;
                    var_count += 1;
                    var_map.insert(name, (Some(id), None));
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
                } => todo!(),
                Stmt::While { condition, body } => todo!(),
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => todo!(),
                Stmt::Block(stmts) => todo!(),
                Stmt::Expression(expr) => todo!(),
                Stmt::Return(expr) => todo!(),
                Stmt::Break => todo!(),
                Stmt::Continue => todo!(),
            }
        }

        return blocks;
    }
}
