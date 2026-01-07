use crate::{
    ir::{
        block::{IRInstruction, StructDef, Value},
        cfg::IRGenerator,
    },
    lexer::ast::{BinaryOp, Expr, Type, UnaryOp},
};

impl IRGenerator {
    fn layout_struct(&self, def: &StructDef) -> Vec<(Type, i32)> {
        // type and offset
        let mut layout = Vec::new();
        if def.is_union {
            for field in def.fields.clone() {
                layout.push((field.1.clone(), 0));
            }
        }

        for field in def.fields.clone() {
            layout.push((field.1.clone(), field.1.size() as i32));
        }
        layout
    }

    pub fn emit_into_local(&mut self, addr: Value, expr: Expr, out: &mut Vec<IRInstruction>) {
        match expr {
            Expr::StructInit { name, params } => {
                let def = self
                    .ir_program
                    .structs
                    .iter()
                    .find(|s| s.name == name)
                    .expect("unknown struct");

                let layout = self.layout_struct(def);

                for (i, (_field_name, field_expr)) in params.iter().enumerate() {
                    let (field_ty, field_off) = &layout[i];

                    let (value, _ty) = self.first_pass_parse_expr(field_expr.clone(), out).unwrap();

                    out.push(IRInstruction::Store {
                        value,
                        addr: addr.clone(),
                        offset: *field_off,
                        ty: field_ty.clone(),
                    });
                }
            }
            other => {
                let (v, ty) = self.first_pass_parse_expr(other, out).unwrap();
                out.push(IRInstruction::Store {
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
                .iter()
                .find(|s| s.name == struct_name)
                .unwrap()
                .fields
                .clone(),
        }
    }

    fn lower_place(&mut self, expr: Expr, _out: &mut Vec<IRInstruction>) -> Option<(Value, Type)> {
        match expr {
            Expr::Variable(name, ty) => {
                let id = self.var_map.get(&name).unwrap().1;
                Some((Value::Local(id), ty))
            }
            Expr::StringLiteral(s) => {
                let g = self.globals.get(&s).unwrap();
                Some((Value::Global(g.id), g.ty.clone()))
            }
            Expr::ArrayAccess { array, index } => None,
            Expr::IndexAssign {
                array,
                index,
                value,
            } => None,
            Expr::InstanceVar(struct_name, field) => {
                let (ty, local_id) = self.var_map.get(&struct_name).unwrap();
                Some((Value::Local(*local_id), ty.clone()))
            }
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => None,
            _ => None, // not an lvalue
        }
    }

    fn first_pass_parse_expr(
        &mut self,
        expr: Expr,
        out: &mut Vec<IRInstruction>,
    ) -> Option<(Value, Type)> {
        let var_map = self.var_map.clone();
        let ir_program = self.ir_program.clone();
        match expr {
            Expr::IntLiteral(i) => Some((Value::Const(i as i64), Type::int)),
            Expr::LongLiteral(l) => Some((Value::Const(l), Type::Long)),
            Expr::FloatLiteral(f) => Some((Value::ConstFloat(f as f64), Type::float)),
            Expr::BoolLiteral(b) => Some((Value::Const(b as i64), Type::Bool)),
            Expr::StringLiteral(s) => {
                let n = self.globals.get(&s).unwrap();
                Some((Value::Global(n.id), n.ty.clone()))
            }
            Expr::CharLiteral(c) => Some((Value::Const(c as i64), Type::Char)),
            Expr::StructInit { name, params } => {
                let def = self
                    .ir_program
                    .structs
                    .iter()
                    .find(|s| s.name == name)
                    .expect("unknown struct");

                let layout = self.layout_struct(def);

                let loc = self.var_gen.fresh();

                for (i, (_field_name, field_expr)) in params.iter().enumerate() {
                    let (field_ty, field_off) = &layout[i];

                    let (value, _ty) = self.first_pass_parse_expr(field_expr.clone(), out).unwrap();

                    out.push(IRInstruction::Store {
                        value,
                        addr: Value::Local(loc),
                        offset: *field_off,
                        ty: field_ty.clone(),
                    });
                }

                Some((Value::Local(loc), self.type_struct(&name)))
            }
            Expr::AddressOf(expression) => {
                let reg = self.vreg_gen.fresh();
                let (place, inner_ty) = self.lower_place(*expression, out).unwrap();
                out.push(IRInstruction::AddressOf {
                    dest: reg,
                    src: place,
                });
                Some((Value::Reg(reg), Type::Pointer(Box::new(inner_ty))))
            }
            Expr::DerefAssign { target, value } => {
                let (rhs_val, _) = self.lower_place(*value, out).unwrap();

                let (ptr_val, ptr_ty) = self.lower_place(*target, out).unwrap(); // ptr_val should be Value::Reg(_)

                let instr = IRInstruction::Store {
                    value: rhs_val,
                    addr: ptr_val,
                    offset: 0,
                    ty: ptr_ty.deref().unwrap().clone(),
                };

                out.push(instr);
                None
            }
            Expr::InstanceVar(_, _) => todo!(),
            Expr::Variable(name, ty) => {
                let id = self
                    .var_map
                    .get(&name)
                    .expect("should not happen :: first_pass_parse_expr :: Expr::Variable")
                    .1;
                let reg = self.vreg_gen.fresh();
                out.push(IRInstruction::Load {
                    reg,
                    addr: Value::Local(id),
                    offset: 0,
                    ty: ty.clone(),
                });
                Some((Value::Reg(reg), ty))
            }
            Expr::Assign { name, value } => {
                let var_info = var_map
                    .get(&name)
                    .expect("should not happen :: first_pass_parse_expr :: Expr::Assign");
                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value, out).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty, out);
                let assign = IRInstruction::Store {
                    value: rhs,
                    addr: Value::Local(var_info.1),
                    offset: 0,
                    ty: var_info.0.clone(),
                };
                out.push(assign);
                None
            }
            Expr::Binary {
                left,
                op,
                right,
                result_type,
            } => {
                let (left_rvalue, left_type) = self.first_pass_parse_expr(*left, out).unwrap();
                let left = self.ensure_rvalue(left_rvalue, &left_type, out);

                let (right_rvalue, right_type) = self.first_pass_parse_expr(*right, out).unwrap();
                let right = self.ensure_rvalue(right_rvalue, &right_type, out);

                let reg = self.vreg_gen.fresh();

                match op {
                    BinaryOp::Add => out.push(IRInstruction::Add { reg, left, right }),
                    BinaryOp::Sub => out.push(IRInstruction::Sub { reg, left, right }),
                    BinaryOp::Mul => out.push(IRInstruction::Mul { reg, left, right }),
                    BinaryOp::Div => out.push(IRInstruction::Div { reg, left, right }),
                    BinaryOp::Mod => out.push(IRInstruction::Mod { reg, left, right }),
                    BinaryOp::Equal => out.push(IRInstruction::Eq { reg, left, right }),
                    BinaryOp::NotEqual => out.push(IRInstruction::Ne { reg, left, right }),
                    BinaryOp::Less => out.push(IRInstruction::Lt { reg, left, right }),
                    BinaryOp::LessEqual => out.push(IRInstruction::Le { reg, left, right }),
                    BinaryOp::Greater => out.push(IRInstruction::Gt { reg, left, right }),
                    BinaryOp::GreaterEqual => out.push(IRInstruction::Ge { reg, left, right }),
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
                    let (v, ty) = self.first_pass_parse_expr(*expr, out).unwrap();
                    let v = self.ensure_rvalue(v, &ty, out);
                    let reg = self.vreg_gen.fresh();
                    out.push(IRInstruction::Eq {
                        reg,
                        left: v,
                        right: Value::Const(0),
                    });
                    Some((Value::Reg(reg), Type::Bool))
                }

                UnaryOp::Negate => {
                    let (v, ty) = self.first_pass_parse_expr(*expr, out).unwrap();
                    let v = self.ensure_rvalue(v, &ty, out);
                    let reg = self.vreg_gen.fresh();
                    out.push(IRInstruction::Sub {
                        reg,
                        left: Value::Const(0),
                        right: v,
                    });
                    Some((Value::Reg(reg), result_type))
                }

                UnaryOp::AddressOf => {
                    let (place, inner_ty) =
                        self.lower_place(*expr, out).expect("cannot take address");
                    let reg = self.vreg_gen.fresh();
                    out.push(IRInstruction::AddressOf {
                        dest: reg,
                        src: place,
                    });
                    Some((Value::Reg(reg), Type::Pointer(Box::new(inner_ty))))
                }

                UnaryOp::Dereference => {
                    let (ptr, ptr_ty) = self.first_pass_parse_expr(*expr, out).unwrap();
                    let ptr = self.ensure_rvalue(ptr, &ptr_ty, out);

                    let pointee = match ptr_ty {
                        Type::Pointer(t) => *t,
                        _ => panic!("dereference of non-pointer"),
                    };

                    let reg = self.vreg_gen.fresh();
                    out.push(IRInstruction::Load {
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
                        let (v, ty) = self.first_pass_parse_expr(arg.clone(), out).unwrap();
                        self.ensure_rvalue(v, &ty, out)
                    })
                    .collect();

                let instr = IRInstruction::Call {
                    reg,
                    func: name,
                    args,
                };
                out.push(instr);
                if let Some(reg) = reg {
                    return Some((Value::Reg(reg), return_type));
                }
                None
            }
            Expr::Cast { expr, target_type } => {
                let register = self.vreg_gen.fresh();
                let (from_val, from_ty) = self.first_pass_parse_expr(*expr, out).unwrap();
                out.push(IRInstruction::Load {
                    reg: register,
                    addr: from_val,
                    offset: 0,
                    ty: from_ty,
                });
                let register2 = self.vreg_gen.fresh();
                out.push(IRInstruction::Cast {
                    reg: register2,
                    src: Value::Reg(register),
                    ty: target_type.clone(),
                });
                Some((Value::Reg(register2), target_type))
            }
            Expr::Array(exprs, ty) => {
                let local_id = self.var_gen.fresh();
                let size_per = ty.size();
                for (i, index_expression) in exprs.iter().enumerate() {
                    let (index_val, _) = self
                        .first_pass_parse_expr(index_expression.clone(), out)
                        .unwrap();
                    let index_val = self.ensure_rvalue(index_val, &ty, out);
                    out.push(IRInstruction::Store {
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
                let register = self.vreg_gen.fresh();

                let (addr_val, mut array_ty) = self.first_pass_parse_expr(*array, out).unwrap();

                if let Type::Array(ty, _) = array_ty {
                    array_ty = *ty;
                }

                let addr_reg = self.vreg_gen.fresh();

                let (offset, ty) = self.first_pass_parse_expr(*index, out).unwrap();
                let offset = self.ensure_rvalue(offset, &ty, out);

                out.push(IRInstruction::Gep {
                    dest: addr_reg,
                    base: addr_val,
                    index: offset,
                    scale: array_ty.size(),
                });

                out.push(IRInstruction::Load {
                    reg: register,
                    addr: Value::Reg(addr_reg),
                    offset: 0,
                    ty: ty.clone(),
                });
                Some((Value::Reg(register), ty))
            }
            Expr::IndexAssign {
                array,
                index,
                value,
            } => {
                let (base, mut base_ty) = self.first_pass_parse_expr(*array, out).unwrap();

                if let Type::Array(elem_ty, _) = base_ty {
                    base_ty = *elem_ty;
                }
                let elem_ty = base_ty.clone();

                let (idx, idx_ty) = self.first_pass_parse_expr(*index, out).unwrap();
                let idx = self.ensure_rvalue(idx, &idx_ty, out);

                let base_ptr = match base {
                    Value::Local(_) | Value::Global(_) => {
                        let r = self.vreg_gen.fresh();
                        out.push(IRInstruction::AddressOf {
                            dest: r,
                            src: base.clone(),
                        });
                        Value::Reg(r)
                    }
                    _ => base.clone(),
                };

                // base_ptr + idx * sizeof(elem)
                let addr_reg = self.vreg_gen.fresh();
                out.push(IRInstruction::Gep {
                    dest: addr_reg,
                    base: base_ptr,
                    index: idx,
                    scale: elem_ty.size(),
                });

                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value, out).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty, out);

                out.push(IRInstruction::Store {
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
                let local_value_of_struct = self.var_map.get(&class_name).unwrap().1;
                let struc = ir_program
                    .structs
                    .iter()
                    .find(|s| s.name == class_name)
                    .unwrap();
                let offset = struc.offsets.get(&field).unwrap();

                let (rhs, rhs_ty) = self.first_pass_parse_expr(*value, out).unwrap();
                let rhs = self.ensure_rvalue(rhs, &rhs_ty, out);

                let instr = IRInstruction::Store {
                    value: rhs,
                    addr: Value::Local(local_value_of_struct),
                    offset: *offset as i32,
                    ty: struc
                        .fields
                        .iter()
                        .find(|s| s.0 == field)
                        .unwrap()
                        .1
                        .clone(),
                };

                out.push(instr);
                None
            }
            Expr::CompoundAssign { name, op, value } => None,
            Expr::PreIncrement { name } => None,
            Expr::PostIncrement { name } => None,
            Expr::PreDecrement { name } => None,
            Expr::PostDecrement { name } => None,
        }
    }

    fn ensure_rvalue(&mut self, v: Value, ty: &Type, out: &mut Vec<IRInstruction>) -> Value {
        match v {
            Value::Local(_) | Value::Global(_) => {
                let r = self.vreg_gen.fresh();
                out.push(IRInstruction::Load {
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
