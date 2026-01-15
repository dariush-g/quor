use std::collections::{HashMap, HashSet, VecDeque};

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

#[derive(Clone, Debug, Default)]
pub struct ScopeHandler {
    pub closed: Vec<BlockId>,
    pub break_stack: VecDeque<BlockId>,
    pub continue_stack: VecDeque<BlockId>,
    pub instructions: Vec<IRInstruction>,
    pub current: BlockId,
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
    pub scope_handler: ScopeHandler,
}

impl IRGenerator {
    pub fn set_current(&mut self, block: BlockId) {
        self.blocks[self.scope_handler.current.0]
            .instructions
            .append(&mut self.scope_handler.instructions);

        self.scope_handler.current = block;
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = self.block_gen.fresh();
        let block = IRBlock {
            id,
            instructions: Vec::new(),
            terminator: Terminator::TemporaryNone,
        };
        self.blocks.push(block);
        id
    }

    // pub fn first_scope(&mut self) -> BlockId {
    //     let id = self.block_gen.fresh();
    //     self.scope_handler.open = id;
    //     id
    // }

    // pub fn new_scope(&mut self) -> BlockId {
    //     let id = self.block_gen.fresh();
    //     self.scope_handler.closed.insert(self.scope_handler.open);
    //     self.scope_handler.open = id;
    //     id
    // }

    pub fn add_new_var(&mut self, name: String, ty: Type) -> Value {
        let id = self.var_gen.fresh();
        self.var_map.insert(name, (ty, id));
        Value::Local(id)
    }

    pub fn generate(stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_generator = IRGenerator::default();
        for stmt in stmts {
            match stmt {
                Stmt::FunDecl { .. } => ir_generator.generate_function(&stmt),
                Stmt::StructDecl { .. } => ir_generator.generate_struct(&stmt),
                Stmt::AtDecl(..) => ir_generator.generate_declaration(&stmt, None),
                _ => Ok(()),
            }?
        }

        Err(String::from("error"))
    }

    fn generate_struct(&mut self, stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_declaration(
        &mut self,
        stmt: &Stmt,
        next_stmt: Option<&Stmt>,
    ) -> Result<(), String> {
        if let Stmt::AtDecl(decl, param, val, content) = stmt {
            let declaration = match decl.as_str() {
                "import" => {
                    if param.clone().unwrap().ends_with("!") {
                        let mut param1 = param.clone().unwrap();
                        param1.pop();
                        AtDecl::Import {
                            path: param1,
                            local: false,
                        }
                    } else {
                        AtDecl::Import {
                            path: param.clone().unwrap(),
                            local: true,
                        }
                    }
                }
                "asm" | "_asm_" | "__asm__" => AtDecl::InlineAssembly {
                    content: param.clone().unwrap(),
                },
                "trust_ret" => AtDecl::TrustRet,
                "define" => AtDecl::Define {
                    name: param.clone().unwrap(),
                    ty: val.clone().unwrap().get_type(),
                    val: val.clone().unwrap(),
                },
                _ => panic!("Unknown AtDecl: {decl}"),
            };
        };
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

            let entry = self.new_block();
            self.set_current(entry);
            self.lower_block(body);

            let ir_func = IRFunction {
                name: name.clone(),
                params,
                ret_type: return_type.clone(),
                blocks: Vec::new(),
                entry,
                attributes: attributes
                    .iter()
                    .map(|att| {
                        AtDecl::parse_attribute(att)
                            .unwrap_or_else(|| panic!("Error parsing IRFunction attributes"))
                    })
                    .collect(),
            };

            self.ir_program.functions.push(ir_func);
        }

        Ok(())
    }
}
