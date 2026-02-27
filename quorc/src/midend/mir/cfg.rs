use std::collections::{HashMap, HashSet, VecDeque};

use crate::{backend::lir::regalloc::RegWidth, frontend::ast::*, midend::mir::block::*};

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
    pub current_offset: i32,
    pub current: BlockId,
}

#[derive(Default)]
pub struct VRegGenerator {
    pub next: usize,
}

impl VRegGenerator {
    pub fn fresh(&mut self, is_float: bool, width: RegWidth) -> VReg {
        let reg = VReg {
            id: self.next,
            ty: if is_float {
                VRegType::Float
            } else {
                VRegType::Int
            },
            width,
        };
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
    pub var_map: HashMap<String, (Type, Value)>,
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
            value: GlobalValue::String(value.clone()),
        };
        self.ir_program.global_consts.insert(id, def.clone());
        self.static_strings.insert(value, def);
    }

    pub fn generate(stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_generator = IRGenerator::default();

        for stmt in stmts.clone() {
            if let Stmt::StructDecl { generics, .. } = &stmt {
                if !generics.is_empty() {
                    continue; // skip generic templates; only concrete monomorphized structs
                }
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

        // ir_generator.generate_function(&Stmt::FunDecl {
        //     name: "malloc".to_string(),
        //     params: vec![("_".to_string(), Type::int)],
        //     return_type: Type::Pointer(Box::new(Type::Void)),
        //     body: Vec::new(),
        //     attributes: Vec::new(),
        // })?;

        ir_generator.blocks = ir_generator
            .blocks
            .clone()
            .iter()
            .filter_map(|block| match block.terminator {
                Terminator::TemporaryNone => None,
                _ => Some(block.clone()),
            })
            .collect::<Vec<_>>();

        Ok(ir_generator.ir_program)
    }

    fn generate_struct(&mut self, stmt: &Stmt) -> Result<(), String> {
        if let Stmt::StructDecl {
            name,
            instances,
            union,
            ..
        } = stmt.clone()
        {
            let offsets = self.get_field_offsets(&instances, union);

            let (max, max_typ) = offsets
                .values()
                .max_by_key(|(offset, _)| *offset)
                .map(|(offset, ty)| (*offset, ty.clone()))
                .unwrap_or((0, Type::Unknown));

            if let Type::Unknown = max_typ {
                let def = StructDef {
                    name: name.clone(),
                    fields: HashMap::new(),
                    is_union: union,
                    size: 0,
                };

                self.ir_program.structs.insert(name, def);
                return Ok(());
            }

            let def = StructDef {
                name: name.clone(),
                fields: offsets,
                is_union: union,
                size: max as usize + max_typ.size(),
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
                "extern" => {
                    self.ir_program.externs.push(param.clone().unwrap());
                }
                "const" => {
                    // let const_value = match val.clone().unwrap() {
                    //     Expr::IntLiteral(i) => GlobalValue::Int(i.into()),
                    //     Expr::LongLiteral(l) => GlobalValue::Int(l),
                    //     Expr::FloatLiteral(f) => GlobalValue::Float(f.into()),
                    //     Expr::BoolLiteral(b) => GlobalValue::Bool(b),
                    //     Expr::StringLiteral(s) => GlobalValue::String(s),
                    //     Expr::CharLiteral(c) => GlobalValue::Char(c),
                    //     Expr::StructInit { .. } => GlobalValue::Struct(val.clone().unwrap()),
                    //     Expr::Array(exprs, _) => {
                    //         GlobalValue::Array(exprs.into_iter().map(|e| {
                    //             match e {
                    //                 Expr::IntLiteral(i) => GlobalValue::Int(i.into()),
                    //                 Expr::LongLiteral(l) => GlobalValue::Int(l),
                    //                 Expr::FloatLiteral(f) => GlobalValue::Float(f.into()),
                    //                 Expr::BoolLiteral(b) => GlobalValue::Bool(b),
                    //                 Expr::StringLiteral(s) => GlobalValue::String(s),
                    //                 Expr::CharLiteral(c) => GlobalValue::Char(c),
                    //                 Expr::StructInit { .. } => GlobalValue::Struct(e),
                    //                 _ => {
                    //                     panic!(
                    //                         "Global constants should only be a single expression of a number, character, string, boolean, array, or struct literal"
                    //                     )
                    //                 }
                    //             }
                    //         }).collect())
                    //     },
                    //     _ => {
                    //         panic!(
                    //             "Global constants should only be a single expression of a number, character, string, boolean, or struct literal"
                    //         )
                    //     }
                    // };
                    let const_value = Self::get_const_value(val.clone().unwrap());
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

    fn get_const_value(expr: Expr) -> GlobalValue {
        match expr {
            Expr::IntLiteral(i) => GlobalValue::Int(i.into()),
            Expr::LongLiteral(l) => GlobalValue::Int(l),
            Expr::FloatLiteral(f) => GlobalValue::Float(f.into()),
            Expr::BoolLiteral(b) => GlobalValue::Bool(b),
            Expr::StringLiteral(s) => GlobalValue::String(s),
            Expr::CharLiteral(c) => GlobalValue::Char(c),
            Expr::StructInit { .. } => GlobalValue::Struct(expr.clone()),
            Expr::Array(exprs, _) => {
                GlobalValue::Array(exprs.into_iter().map(Self::get_const_value).collect())
            }
            _ => {
                panic!(
                    "Global constants should only be a single expression of a number, character, string, or boolean literal"
                )
            }
        }
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
            let entry = self.new_block();
            // Set current directly instead of using set_current() to avoid
            // leaking the previous function's last block into this function's
            // closed set.
            self.scope_handler.current = entry;

            let mut params = Vec::with_capacity(func_params.len());
            for (param_name, param_ty) in func_params.clone() {
                let param_reg = self
                    .vreg_gen
                    .fresh(Type::float == param_ty, type_to_reg_width(&param_ty));
                if param_ty.fits_in_register() {
                    // Primitive or pointer: keep in VReg, no stack allocation needed
                    self.var_map
                        .insert(param_name, (param_ty.clone(), Value::Reg(param_reg)));
                } else if let Type::Struct {
                    name: struct_name, ..
                } = &param_ty
                {
                    // Struct: allocate stack slot and copy from param (param_reg holds address)
                    let local = self.var_gen.fresh();
                    self.var_map
                        .insert(param_name.clone(), (param_ty.clone(), Value::Local(local)));
                    self.allocate_struct_on_stack(
                        Value::Local(local),
                        param_reg,
                        struct_name.clone(),
                    );
                // } else {
                //     // array or other: allocate stack slot and copy
                //     let local = self.var_gen.fresh();
                //     self.var_map
                //         .insert(param_name.clone(), (param_ty.clone(), Value::Local(local)));

                //     self.scope_handler.instructions.push(IRInstruction::Store {
                //         value: Value::Reg(param_reg),
                //         addr: Value::Local(local),
                //         offset: 0,
                //         ty: param_ty,
                //     });
                // }
                } else {
                    let local = self.var_gen.fresh();
                    self.var_map
                        .insert(param_name.clone(), (param_ty.clone(), Value::Local(local)));

                    let src_addr = Value::Reg(param_reg);
                    let dst_addr = Value::Local(local);

                    self.scope_handler.instructions.push(IRInstruction::Memcpy {
                        dst: dst_addr,
                        src: src_addr,
                        size: param_ty.size(),
                        align: param_ty.align(),
                    });
                }

                params.push(param_reg);
            }

            // println!("inside block id: {:?}", self.scope_handler.current);

            self.lower_block(body);

            self.blocks[self.scope_handler.current.0]
                .instructions
                .append(&mut self.scope_handler.instructions);

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

            let offset = self.scope_handler.current_offset;
            self.scope_handler.current_offset = 0;

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
                offset,
            };

            self.var_map = HashMap::new();
            self.ir_program.functions.insert(name.to_string(), ir_func);
        }

        Ok(())
    }
}
