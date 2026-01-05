use std::collections::HashMap;

use crate::{
    ir::{
        block::{GlobalDef, IRInstruction, IRProgram, VReg, Value},
        cfg::IRGenerator,
    },
    lexer::ast::{Expr, Type},
};

impl IRGenerator {
    fn first_pass_parse_expr(
        &mut self,
        expr: Expr,
        ir_program: &mut IRProgram,
        var_map: &HashMap<String, (Option<usize>, Option<VReg>)>,
        var_count: usize,
    ) -> IRInstruction {
        let (value, ty) = match expr {
            Expr::IntLiteral(i) => (Value::Const(i as i64), Type::int),
            Expr::LongLiteral(l) => (Value::Const(l), Type::Long),
            Expr::FloatLiteral(f) => (Value::ConstFloat(f as f64), Type::float),
            Expr::BoolLiteral(b) => (Value::Const(b as i64), Type::Bool),
            Expr::StringLiteral(s) => {
                todo!()
            }
            Expr::CharLiteral(_) => todo!(),
            Expr::StructInit { name, params } => todo!(),
            Expr::AddressOf(expr) => todo!(),
            Expr::DerefAssign { target, value } => todo!(),
            Expr::InstanceVar(_, _) => todo!(),
            Expr::Variable(_, _) => todo!(),
            Expr::Assign { name, value } => todo!(),
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
        };

        let count = var_count;
        var_count += 1;

        IRInstruction::Store {
            value,
            addr: Value::Local(count),
            offset: 0,
            ty,
        }
    }
}
