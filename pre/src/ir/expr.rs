use std::collections::HashMap;

use crate::{
    ir::{
        block::{GlobalDef, IRInstruction, IRProgram, StructDef, VReg, Value},
        cfg::IRGenerator,
    },
    lexer::ast::{Expr, Type},
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

    fn lower_place(&mut self, expr: Expr, out: &mut Vec<IRInstruction>) -> Option<Value> {
        match expr {
            Expr::Variable(name, _ty) => {
                let id = self.var_map.get(&name).unwrap().1;
                Some(Value::Local(id))
            }
            Expr::StringLiteral(s) => {
                let g = self.globals.get(&s).unwrap();
                Some(Value::Global(g.id))
            }
            // Expr::Deref(inner) => {
            //     // if you have deref lvalues: *p is a place whose address is just p
            //     // (i.e. *p as a place is "Reg(ptr)" for your Store/Load addr field)
            //     let (ptr_val, _ty) = self.first_pass_parse_expr(*inner, out).unwrap();
            //     Some(ptr_val) // must be Value::Reg(...) typically
            // }
            Expr::ArrayAccess { array, index } => {
                // compute element address with Gep and return Value::Reg(addr)
                // requires types + scale; return that computed address as a "place"
                todo!()
            }
            Expr::InstanceVar(obj, field) => {
                // compute field address and return it as a "place"
                todo!()
            }
            _ => None, // not an lvalue
        }
    }

    fn first_pass_parse_expr(
        &mut self,
        expr: Expr,
        out: &mut Vec<IRInstruction>,
    ) -> Option<(Value, Type)> {
        let var_map = self.var_map.clone();
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
                let load = IRInstruction::AddressOf {
                    dest: self.vreg_gen.fresh(),
                    src: self.first_pass_parse_expr(*expression, out).unwrap().0,
                };
                out.push(load);
                None
            }
            Expr::DerefAssign { target, value } => todo!(),
            Expr::InstanceVar(_, _) => todo!(),
            Expr::Variable(name, ty) => {
                let id = self
                    .var_map
                    .get(&name)
                    .expect("should not happen :: first_pass_parse_expr :: Expr::Variable")
                    .1;
                Some((Value::Local(id), ty))
            }
            Expr::Assign { name, value } => {
                let var_info = var_map
                    .get(&name)
                    .expect("should not happen :: first_pass_parse_expr :: Expr::Assign");
                let val = self.first_pass_parse_expr(*value, out).unwrap().0;
                let assign = IRInstruction::Store {
                    value: Value::Local(var_info.1),
                    addr: val,
                    offset: 0,
                    ty: var_info.0.clone(),
                };
                out.push(assign);
                None
            }
            Expr::CompoundAssign { name, op, value } => todo!(),
            Expr::PreIncrement { name } => todo!(),
            Expr::PostIncrement { name } => todo!(),
            Expr::PreDecrement { name } => todo!(),
            Expr::PostDecrement { name } => todo!(),
            Expr::Binary {
                left,
                op,
                right,
                result_type,
            } => todo!(),
            Expr::Unary {
                op,
                expr,
                result_type,
            } => todo!(),
            Expr::Call {
                name,
                args,
                return_type,
            } => todo!(),
            Expr::Cast { expr, target_type } => todo!(),
            Expr::Array(exprs, _) => todo!(),
            Expr::ArrayAccess { array, index } => todo!(),
            Expr::IndexAssign {
                array,
                index,
                value,
            } => todo!(),
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => todo!(),
        }
    }
}
