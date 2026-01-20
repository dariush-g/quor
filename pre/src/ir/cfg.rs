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
    pub closed: HashSet<BlockId>,
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

#[derive(Debug, Default, Clone)]
pub struct GlobalGenerator {
    pub next: usize,
}

impl GlobalGenerator {
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
    pub global_gen: GlobalGenerator,
    pub var_map: HashMap<String, (Type, usize)>,
    pub blocks: Vec<IRBlock>,
    pub globals: HashMap<String, GlobalDef>,
    pub static_strings: HashMap<String, GlobalDef>,
    pub ir_program: IRProgram,
    pub scope_handler: ScopeHandler,
}

impl IRGenerator {
    pub fn set_current(&mut self, block: BlockId) {
        self.blocks[self.scope_handler.current.0]
            .instructions
            .append(&mut self.scope_handler.instructions);

        self.scope_handler.closed.insert(self.scope_handler.current);
        self.scope_handler.current = block;
    }

    pub fn new_block(&mut self) -> BlockId {
        let id = self.block_gen.fresh();
        let block = IRBlock {
            id,
            instructions: Vec::new(),
            terminator: Terminator::TemporaryNone,
        };
        self.scope_handler.closed.insert(id);
        self.blocks.push(block);
        id
    }

    pub fn new_global(&mut self, name: String, ty: Type, value: GlobalValue) {
        let id = self.global_gen.fresh();
        let def = GlobalDef { id, ty, value };
        self.globals.insert(name, def.clone());
        self.ir_program.global_consts.push(def);
    }

    pub fn new_static_string(&mut self, value: String) {
        let id = self.global_gen.fresh();
        let def = GlobalDef {
            id,
            ty: Type::Pointer(Box::new(Type::Char)),
            value: GlobalValue::Bytes(value.as_bytes().to_vec()),
        };
        self.static_strings.insert(value, def);
    }

    pub fn add_new_var(&mut self, name: String, ty: Type) -> Value {
        let id = self.var_gen.fresh();
        self.var_map.insert(name, (ty, id));
        Value::Local(id)
    }

    pub fn generate(stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_generator = IRGenerator::default();

        for stmt in stmts.clone() {
            if let Stmt::StructDecl { .. } = stmt {
                ir_generator.generate_struct(&stmt)?
            }
        }

        // println!("{:?}", ir_generator.ir_program.structs);

        for stmt in stmts {
            match stmt {
                Stmt::FunDecl { .. } => ir_generator.generate_function(&stmt)?,
                Stmt::AtDecl(..) => ir_generator.generate_declaration(&stmt, None)?,
                _ => {}
            }
        }

        Ok(ir_generator.ir_program)
    }

    fn generate_struct(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::StructDecl {
            name,
            instances,
            union,
        } = stmt.clone()
        {
            let offsets = self.get_field_offsets(&instances, union);

            let def = StructDef {
                name: name.clone(),
                fields: offsets,
                is_union: union,
            };

            self.ir_program.structs.insert(name, def);
        }

        Ok(())
    }

    fn generate_declaration(
        &mut self,
        stmt: &Stmt,
        _next_stmt: Option<&Stmt>,
    ) -> Result<(), String> {
        if let Stmt::AtDecl(decl, param, val, _content) = stmt {
            match decl.as_str() {
                "import" => {
                    // let path = param.as_ref().unwrap();
                    // if path.ends_with('!') {
                    //     let mut path = path.clone();
                    //     path.pop();
                    //     self.ir_program.imports.push((path, false));
                    // } else {
                    //     self.ir_program.imports.push((path.clone(), true));
                    // }
                }
                "define" => {
                    let const_value = match val.clone().unwrap() {
                        Expr::IntLiteral(i) => GlobalValue::Int(i.into()),
                        Expr::LongLiteral(l) => GlobalValue::Int(l),
                        Expr::FloatLiteral(f) => GlobalValue::Float(f.into()),
                        Expr::BoolLiteral(b) => GlobalValue::Bool(b),
                        Expr::StringLiteral(s) => GlobalValue::Bytes(s.as_bytes().to_vec()),
                        Expr::CharLiteral(c) => GlobalValue::Char(c),
                        Expr::StructInit { .. } => GlobalValue::Struct(val.clone().unwrap()),
                        _ => {
                            panic!(
                                "Global constants should only be a single expression of a number, character, string, boolean, or struct literal"
                            )
                        }
                    };
                    self.new_global(
                        param.clone().unwrap(),
                        val.as_ref().unwrap().get_type(),
                        const_value,
                    );
                }
                _ => {}
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

            let mut blocks: Vec<IRBlock> = self
                .scope_handler
                .closed
                .iter()
                .map(|id| self.blocks[id.0].clone())
                .collect();
            
            self.scope_handler.closed = HashSet::new();

            if let Some(last) = blocks.last_mut()
                && *return_type == Type::Void
                && let Terminator::TemporaryNone = last.terminator
            {
                last.terminator = Terminator::Return { value: None };
            }

            let ir_func = IRFunction {
                name: name.clone(),
                params,
                ret_type: return_type.clone(),
                blocks,
                entry,
                attributes: attributes
                    .iter()
                    .filter_map(|attr| AtDecl::parse_attribute(attr.as_str()))
                    .collect(),
            };

            self.ir_program.functions.insert(name.to_string(), ir_func);
        }

        Ok(())
    }
}
