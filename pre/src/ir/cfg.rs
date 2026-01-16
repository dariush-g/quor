use std::collections::{HashMap, VecDeque};

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

    pub fn add_new_var(&mut self, name: String, ty: Type) -> Value {
        let id = self.var_gen.fresh();
        self.var_map.insert(name, (ty, id));
        Value::Local(id)
    }

    pub fn generate(stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_generator = IRGenerator::default();

        for stmt in stmts {
            if let Stmt::Expression(expr) = stmt {
                if let Expr::StringLiteral(string) = expr {
                    let global_def = GlobalDef {
                        id: ,
                        ty: Type::Pointer(Box::new(Type::Char)),
                        value: GlobalValue::Bytes(string.bytes()),
                    };
                    ir_generator.globals.insert(string, v)
                }
            }
        }

        for stmt in stmts {
            match stmt {
                Stmt::FunDecl { .. } => ir_generator.generate_function(&stmt)?,
                Stmt::StructDecl { .. } => ir_generator.generate_struct(&stmt)?,
                Stmt::AtDecl(..) => ir_generator.generate_declaration(&stmt, None)?,
                _ => {}
            }
        }

        Ok(ir_generator.ir_program)
    }

    fn generate_struct(&mut self, _stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_declaration(
        &mut self,
        stmt: &Stmt,
        _next_stmt: Option<&Stmt>,
    ) -> Result<(), String> {
        if let Stmt::AtDecl(decl, param, val, _content) = stmt {
            let _declaration = match decl.as_str() {
                "import" => {
                    let path = param.as_ref().unwrap();
                    if path.ends_with('!') {
                        let mut path = path.clone();
                        path.pop();
                        AtDecl::Import { path, local: false }
                    } else {
                        AtDecl::Import {
                            path: path.clone(),
                            local: true,
                        }
                    }
                }
                "asm" | "_asm_" | "__asm__" => AtDecl::InlineAssembly {
                    content: param.as_ref().unwrap().clone(),
                },
                "trust_ret" => AtDecl::TrustRet,
                "define" => AtDecl::Define {
                    name: param.as_ref().unwrap().clone(),
                    ty: val.as_ref().unwrap().get_type(),
                    val: val.as_ref().unwrap().clone(),
                },
                _ => panic!("Unknown AtDecl: {decl}"),
            };
        }
        Ok(())
    }

    fn generate_function(&mut self, func: &Stmt) -> Result<(), String> {
        if let Stmt::FunDecl {
            name,
            params: func_params,
            return_type,
            body,
            attributes,
        } = func
        {
            let mut params = Vec::with_capacity(func_params.len());
            for _ in 0..func_params.len() {
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
                    .filter_map(|attr| AtDecl::parse_attribute(attr.as_str()))
                    .collect(),
            };

            self.ir_program.functions.push(ir_func);
        }

        Ok(())
    }
}
