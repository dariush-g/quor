use std::collections::HashMap;

use crate::{
    ir::{
        block::{GlobalValue, IRInstruction, VReg, Value},
        cfg::IRGenerator,
    },
    lexer::ast::{BinaryOp, Expr, Type, UnaryOp},
};

impl IRGenerator {
    pub fn allocate_struct_on_stack(&mut self, local: Value, param_reg: VReg, struct_name: String) {
        let fields = {
            let struct_def = self
                .ir_program
                .structs
                .get(&struct_name)
                .expect("struct definition not found for parameter");
            struct_def.fields.clone()
        };

        self.copy_struct_fields(Value::Reg(param_reg), local, &fields);
    }

    fn copy_struct_fields(
        &mut self,
        src: Value,
        dst: Value,
        fields: &HashMap<String, (i32, Type)>,
    ) {
        self.copy_struct_fields_with_base_offset(src, dst, fields, 0);
    }

    fn copy_struct_fields_with_base_offset(
        &mut self,
        src: Value,
        dst: Value,
        fields: &HashMap<String, (i32, Type)>,
        base_offset: i32,
    ) {
        for (field_offset, field_ty) in fields.values() {
            let total_offset = base_offset + *field_offset;

            match field_ty {
                Type::Struct {
                    name: nested_struct_name,
                    ..
                } => {
                    let nested_fields = {
                        let nested_struct_def = self
                            .ir_program
                            .structs
                            .get(nested_struct_name)
                            .expect("nested struct definition not found");
                        nested_struct_def.fields.clone()
                    };

                    self.copy_struct_fields_with_base_offset(
                        src.clone(),
                        dst.clone(),
                        &nested_fields,
                        total_offset,
                    );
                }
                _ => {
                    let temp_reg = self.vreg_gen.fresh();

                    self.scope_handler.instructions.push(IRInstruction::Load {
                        reg: temp_reg,
                        addr: src.clone(),
                        offset: total_offset,
                        ty: field_ty.clone(),
                    });

                    self.scope_handler.instructions.push(IRInstruction::Store {
                        value: Value::Reg(temp_reg),
                        addr: dst.clone(),
                        offset: total_offset,
                        ty: field_ty.clone(),
                    });
                }
            }
        }
    }

    pub fn get_field_offsets(
        &self,
        fields: &Vec<(String, Type)>,
        is_union: bool,
    ) -> HashMap<String, (i32, Type)> {
        let mut map = HashMap::new();
        if is_union {
            for field in fields {
                map.insert(field.0.clone(), (0, field.1.clone()));
            }
        } else {
            let mut offset_compound = 0;
            for field in fields {
                map.insert(field.0.clone(), (offset_compound as i32, field.1.clone()));
                offset_compound += field.1.size();
            }
        }

        map
    }

    pub fn emit_into_local(&mut self, addr: Value, expr: Expr) {
        match expr {
            Expr::StructInit { name, params } => {
                let def = self
                    .ir_program
                    .structs
                    .get(&name)
                    .expect("no known struct: '{name}'");

                let layout = def.fields.clone();

                for (field_name, field_expr) in params.iter() {
                    let (field_off, field_ty) = &layout.get(field_name).unwrap();

                    let (value, _ty) = self.first_pass_parse_expr(field_expr.clone()).unwrap();

                    self.scope_handler.instructions.push(IRInstruction::Store {
                        value,
                        addr: addr.clone(),
                        offset: *field_off,
                        ty: field_ty.clone(),
                    });
                }
            }
            other => {
                let (v, ty) = self.first_pass_parse_expr(other).unwrap();
                self.scope_handler.instructions.push(IRInstruction::Store {
                    value: v,
                    addr,
                    offset: 0,
                    ty,
                });
            }
        }
    }

    fn type_struct(&self, struct_name: &str) -> Type {
        Type::Struct {
            name: struct_name.to_owned(),
            instances: self
                .ir_program
                .structs
                .get(struct_name)
                .expect("no known struct")
                .fields
                .clone()
                .iter()
                .map(|f| (f.0.clone(), f.1.1.clone()))
                .collect(),
        }
    }

    pub fn lower_place(&mut self, expr: Expr) -> Option<(Value, Type)> {
        match expr {
            Expr::BoolLiteral(b) => Some((Value::Const(b as i64), Type::Bool)),
            Expr::IntLiteral(i) => Some((Value::Const(i as i64), Type::Bool)),
            Expr::LongLiteral(i) => Some((Value::Const(i), Type::Bool)),
            Expr::Variable(name, ty) => {
                let id = self.var_map.get(&name).unwrap().1;
                Some((Value::Local(id), ty))
            }
            Expr::StringLiteral(s) => {
                let mut g = self.static_strings.get(&s);
                if g.is_none() {
                    self.new_static_string(s.clone());
                    g = self.static_strings.get(&s);
                }
                let g = g.unwrap();
                Some((Value::Global(g.id), g.ty.clone()))
            }
            Expr::ArrayAccess { array: _, index: _ } => None,
            Expr::IndexAssign {
                array: _,
                index: _,
                value: _,
            } => None,
            Expr::InstanceVar(struct_name, _field) => {
                let (ty, local_id) = self.var_map.get(&struct_name).unwrap();
                Some((Value::Local(*local_id), ty.clone()))
            }
            Expr::FieldAssign {
                class_name: _,
                field: _,
                value: _,
            } => None,
            _ => None, // not an lvalue
        }
    }

    pub fn first_pass_parse_expr(&mut self, expr: Expr) -> Option<(Value, Type)> {
        match expr {
            Expr::IntLiteral(i) => Some((Value::Const(i as i64), Type::int)),
            Expr::LongLiteral(l) => Some((Value::Const(l), Type::Long)),
            Expr::FloatLiteral(f) => Some((Value::ConstFloat(f as f64), Type::float)),
            Expr::BoolLiteral(b) => Some((Value::Const(b as i64), Type::Bool)),
            Expr::StringLiteral(s) => {
                let mut def = self.globals.get(&s);
                if def.is_none() {
                    self.new_static_string(s.clone());
                    def = self.static_strings.get(&s);
                }
                let def = def.unwrap();
                Some((Value::Global(def.id), def.ty.clone()))
            }
            Expr::CharLiteral(c) => Some((Value::Const(c as i64), Type::Char)),
            Expr::StructInit { name, params } => {
                let def = self.ir_program.structs.get(&name).expect("unknown struct");

                let layout = def.fields.clone();

                let loc = self.var_gen.fresh();

                for (field_name, field_expr) in params.iter() {
                    let (field_off, field_ty) = &layout.get(field_name).unwrap();

                    let (value, _ty) = self.first_pass_parse_expr(field_expr.clone()).unwrap();

                    self.scope_handler.instructions.push(IRInstruction::Store {
                        value,
                        addr: Value::Local(loc),
                        offset: *field_off,
                        ty: field_ty.clone(),
                    });
                }

                Some((Value::Local(loc), self.type_struct(&name)))
            }
            Expr::AddressOf(expression) => {
                let (place, inner_ty) = self.lower_place(*expression).unwrap();
                let reg = self.vreg_gen.fresh();
                self.scope_handler
                    .instructions
                    .push(IRInstruction::AddressOf {
                        dest: reg,
                        src: place,
                    });
                Some((Value::Reg(reg), Type::Pointer(Box::new(inner_ty))))
            }
            Expr::DerefAssign { target, value } => {
                let (rhs_val, _) = self.lower_place(*value).unwrap();
                let (ptr_val, ptr_ty) = self.lower_place(*target).unwrap();
                let pointee_ty = ptr_ty
                    .deref()
                    .expect("cannot dereference non-pointer")
                    .clone();

                self.scope_handler.instructions.push(IRInstruction::Store {
                    value: rhs_val,
                    addr: ptr_val,
                    offset: 0,
                    ty: pointee_ty,
                });
                None
            }
            Expr::InstanceVar(struct_var_name, field_name) => {
                let mut field_type = Type::Void;
                let vreg = self.vreg_gen.fresh();
                if let Some((ty, id)) = self.var_map.get(&struct_var_name) {
                    if let Type::Struct { name, .. } = ty {
                        let struct_def = self.ir_program.structs.get(name).unwrap();
                        let offset = struct_def.fields.get(&field_name).unwrap().0;
                        field_type = struct_def.fields.get(&field_name).unwrap().1.clone();
                        self.scope_handler.instructions.push(IRInstruction::Load {
                            reg: vreg,
                            addr: Value::Local(*id),
                            offset,
                            ty: field_type.clone(),
                        });
                    }
                } else if let Some(global_def) = self.globals.get(&struct_var_name)
                    && let GlobalValue::Struct(expr) = &global_def.value
                    && let Expr::StructInit { name, .. } = expr
                {
                    let struct_def = self.ir_program.structs.get(name).unwrap();
                    let offset = struct_def.fields.get(&field_name).unwrap().0;
                    field_type = struct_def.fields.get(&field_name).unwrap().1.clone();
                    self.scope_handler.instructions.push(IRInstruction::Load {
                        reg: vreg,
                        addr: Value::Global(global_def.id),
                        offset,
                        ty: field_type.clone(),
                    });
                }

                Some((Value::Reg(vreg), field_type))
            }
            Expr::Variable(name, ty) => {
                let id = match self.var_map.get(&name) {
                    Some(var_info) => var_info.1,
                    None => self.globals.get(&name).expect("variable not found").id,
                };
                let reg = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Load {
                    reg,
                    addr: Value::Local(id),
                    offset: 0,
                    ty: ty.clone(),
                });
                Some((Value::Reg(reg), ty))
            }
            Expr::Assign { name, value } => {
                let (local_id, var_ty) = {
                    let var_info = self
                        .var_map
                        .get(&name)
                        .expect("variable not found in assignment");
                    (var_info.1, var_info.0.clone())
                };
                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty);
                self.scope_handler.instructions.push(IRInstruction::Store {
                    value: rhs,
                    addr: Value::Local(local_id),
                    offset: 0,
                    ty: var_ty,
                });
                None
            }
            Expr::Binary {
                left,
                op,
                right,
                result_type,
            } => {
                let (left_rvalue, left_type) = self.first_pass_parse_expr(*left).unwrap();
                let left = self.ensure_rvalue(left_rvalue, &left_type);

                let (right_rvalue, right_type) = self.first_pass_parse_expr(*right).unwrap();
                let right = self.ensure_rvalue(right_rvalue, &right_type);

                let reg = self.vreg_gen.fresh();

                match op {
                    BinaryOp::Add => self.scope_handler.instructions.push(IRInstruction::Add {
                        reg,
                        left,
                        right,
                    }),
                    BinaryOp::Sub => self.scope_handler.instructions.push(IRInstruction::Sub {
                        reg,
                        left,
                        right,
                    }),
                    BinaryOp::Mul => self.scope_handler.instructions.push(IRInstruction::Mul {
                        reg,
                        left,
                        right,
                    }),
                    BinaryOp::Div => self.scope_handler.instructions.push(IRInstruction::Div {
                        reg,
                        left,
                        right,
                    }),
                    BinaryOp::Mod => self.scope_handler.instructions.push(IRInstruction::Mod {
                        reg,
                        left,
                        right,
                    }),
                    BinaryOp::Equal => {
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::Eq { reg, left, right })
                    }
                    BinaryOp::NotEqual => {
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::Ne { reg, left, right })
                    }
                    BinaryOp::Less => {
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::Lt { reg, left, right })
                    }
                    BinaryOp::LessEqual => self
                        .scope_handler
                        .instructions
                        .push(IRInstruction::Le { reg, left, right }),
                    BinaryOp::Greater => {
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::Gt { reg, left, right })
                    }
                    BinaryOp::GreaterEqual => self
                        .scope_handler
                        .instructions
                        .push(IRInstruction::Ge { reg, left, right }),
                    // BinaryOp::AND => todo!(),
                    // BinaryOp::OR => todo!(),
                    // BinaryOp::XOR => todo!(),
                    // BinaryOp::NOT => todo!(),
                    // BinaryOp::LSHIFT => todo!(),
                    // BinaryOp::RSHIFT => todo!(),
                    // BinaryOp::ZFILLRSHIFT => todo!(),
                    // BinaryOp::And => todo!(),
                    // BinaryOp::Or => todo!(),
                    _ => {}
                }

                Some((Value::Reg(reg), result_type))
            }
            Expr::Unary {
                op,
                expr,
                result_type,
            } => match op {
                UnaryOp::Not => {
                    let (v, ty) = self.first_pass_parse_expr(*expr).unwrap();
                    let v = self.ensure_rvalue(v, &ty);
                    let reg = self.vreg_gen.fresh();
                    self.scope_handler.instructions.push(IRInstruction::Eq {
                        reg,
                        left: v,
                        right: Value::Const(0),
                    });
                    Some((Value::Reg(reg), Type::Bool))
                }

                UnaryOp::Negate => {
                    let (v, ty) = self.first_pass_parse_expr(*expr).unwrap();
                    let v = self.ensure_rvalue(v, &ty);
                    let reg = self.vreg_gen.fresh();
                    self.scope_handler.instructions.push(IRInstruction::Sub {
                        reg,
                        left: Value::Const(0),
                        right: v,
                    });
                    Some((Value::Reg(reg), result_type))
                }

                UnaryOp::AddressOf => {
                    let (place, inner_ty) = self
                        .lower_place(*expr)
                        .expect("cannot take address of non-place");
                    let reg = self.vreg_gen.fresh();
                    self.scope_handler
                        .instructions
                        .push(IRInstruction::AddressOf {
                            dest: reg,
                            src: place,
                        });
                    Some((Value::Reg(reg), Type::Pointer(Box::new(inner_ty))))
                }

                UnaryOp::Dereference => {
                    let (ptr, ptr_ty) = self.first_pass_parse_expr(*expr).unwrap();
                    let ptr = self.ensure_rvalue(ptr, &ptr_ty);

                    let pointee = match ptr_ty {
                        Type::Pointer(t) => *t,
                        _ => panic!("cannot dereference non-pointer type"),
                    };

                    let reg = self.vreg_gen.fresh();
                    self.scope_handler.instructions.push(IRInstruction::Load {
                        reg,
                        addr: ptr,
                        offset: 0,
                        ty: pointee.clone(),
                    });
                    Some((Value::Reg(reg), pointee))
                }
            },
            Expr::Call {
                name,
                args,
                return_type,
            } => {
                let reg = if return_type == Type::Void {
                    None
                } else {
                    Some(self.vreg_gen.fresh())
                };

                let args: Vec<Value> = args
                    .iter()
                    .map(|arg| {
                        let (v, ty) = self.first_pass_parse_expr(arg.clone()).unwrap();
                        self.ensure_rvalue(v, &ty)
                    })
                    .collect();

                let instr = IRInstruction::Call {
                    reg,
                    func: name,
                    args,
                };

                self.scope_handler.instructions.push(instr);

                if let Some(reg) = reg {
                    return Some((Value::Reg(reg), return_type));
                }
                None
            }
            Expr::Cast { expr, target_type } => {
                let (from_val, from_ty) = self.first_pass_parse_expr(*expr).unwrap();
                let from_val = self.ensure_rvalue(from_val, &from_ty);
                let result_reg = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Cast {
                    reg: result_reg,
                    src: from_val,
                    ty: target_type.clone(),
                });
                Some((Value::Reg(result_reg), target_type))
            }
            Expr::Array(exprs, ty) => {
                let local_id = self.var_gen.fresh();
                let size_per = ty.size();
                for (i, index_expression) in exprs.iter().enumerate() {
                    let (index_val, _) = self
                        .first_pass_parse_expr(index_expression.clone())
                        .unwrap();
                    let index_val = self.ensure_rvalue(index_val, &ty);
                    self.scope_handler.instructions.push(IRInstruction::Store {
                        value: index_val,
                        addr: Value::Local(local_id),
                        offset: (i * size_per) as i32,
                        ty: ty.clone(),
                    });
                }
                Some((
                    Value::Local(local_id),
                    Type::Array(Box::new(ty), Some(exprs.len())),
                ))
            }
            Expr::ArrayAccess { array, index } => {
                let (addr_val, array_ty) = self.first_pass_parse_expr(*array).unwrap();
                let elem_ty = if let Type::Array(ty, _) = array_ty {
                    *ty
                } else {
                    array_ty
                };

                let (index_val, index_ty) = self.first_pass_parse_expr(*index).unwrap();
                let index_val = self.ensure_rvalue(index_val, &index_ty);

                let addr_reg = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Gep {
                    dest: addr_reg,
                    base: addr_val,
                    index: index_val,
                    scale: elem_ty.size(),
                });

                let result_reg = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Load {
                    reg: result_reg,
                    addr: Value::Reg(addr_reg),
                    offset: 0,
                    ty: elem_ty.clone(),
                });
                Some((Value::Reg(result_reg), elem_ty))
            }
            Expr::IndexAssign {
                array,
                index,
                value,
            } => {
                let (base, base_ty) = self.first_pass_parse_expr(*array).unwrap();
                let elem_ty = if let Type::Array(ty, _) = base_ty {
                    *ty
                } else {
                    base_ty
                };

                let (idx, idx_ty) = self.first_pass_parse_expr(*index).unwrap();
                let idx = self.ensure_rvalue(idx, &idx_ty);

                let base_ptr = match base {
                    Value::Local(_) | Value::Global(_) => {
                        let reg = self.vreg_gen.fresh();
                        self.scope_handler
                            .instructions
                            .push(IRInstruction::AddressOf {
                                dest: reg,
                                src: base,
                            });
                        Value::Reg(reg)
                    }
                    _ => base,
                };

                let addr_reg = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Gep {
                    dest: addr_reg,
                    base: base_ptr,
                    index: idx,
                    scale: elem_ty.size(),
                });

                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty);

                self.scope_handler.instructions.push(IRInstruction::Store {
                    value: rhs,
                    addr: Value::Reg(addr_reg),
                    offset: 0,
                    ty: elem_ty,
                });

                None
            }
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => {
                let var_map = self.var_map.clone();

                let (local_id, offset, field_ty) = {
                    let (struct_type, local_id) =
                        var_map.get(&class_name).expect("struct variable not found");

                    if let Type::Struct { name, .. } = struct_type {
                        let struc = self.ir_program.structs.get(name).expect("struct not found");

                        let offset = struc
                            .fields
                            .get(&field)
                            .expect("field not found in struct")
                            .0;
                        let field_ty = struc.fields.get(&field).unwrap().1.clone();

                        Some((local_id, offset, field_ty))
                    } else {
                        None
                    }
                }
                .unwrap();

                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty);

                self.scope_handler.instructions.push(IRInstruction::Store {
                    value: rhs,
                    addr: Value::Local(*local_id),
                    offset,
                    ty: field_ty,
                });
                None
            }
            Expr::CompoundAssign {
                name: _,
                op: _,
                value: _,
            } => None,
            Expr::PreIncrement { name: _ } => None,
            Expr::PostIncrement { name: _ } => None,
            Expr::PreDecrement { name: _ } => None,
            Expr::PostDecrement { name: _ } => None,
        }
    }

    fn ensure_rvalue(&mut self, v: Value, ty: &Type) -> Value {
        match v {
            Value::Local(_) | Value::Global(_) => {
                let r = self.vreg_gen.fresh();
                self.scope_handler.instructions.push(IRInstruction::Load {
                    reg: r,
                    addr: v,
                    offset: 0,
                    ty: ty.clone(),
                });
                Value::Reg(r)
            }
            _ => v,
        }
    }
}
