use std::collections::HashMap;

use crate::lexer::{
    ast::{Expr, Type},
    token::{FloatType, IntType},
};

struct TypeChecker {
    variables: Vec<HashMap<String, Type>>, // stack of scopes
    functions: HashMap<String, (Vec<Type>, Type)>, // fn_name -> (param types, return type)
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn declare_var(&mut self, name: &str, ty: Type) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ty);
    }

    fn lookup_var(&mut self, name: &str) -> Option<&Type> {
        for scope in self.variables.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn declare_fn(&mut self, name: &str, params: Vec<Type>, ret: Type) {
        self.functions.insert(name.to_string(), (params, ret));
    }

    fn lookup_fn(&self, name: &str) -> Option<&(Vec<Type>, Type)> {
        self.functions.get(name)
    }

    fn type_check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IntLiteral(_, ty) => {
                // map IntType to Type
                match ty {
                    IntType::i8 => Ok(Type::i8),
                    IntType::i16 => Ok(Type::i16),
                    IntType::i32 => Ok(Type::i32),
                    IntType::i64 => Ok(Type::i64),
                    IntType::i128 => Ok(Type::i128),
                    IntType::u8 => Ok(Type::u8),
                    IntType::u16 => Ok(Type::u16),
                    IntType::u32 => Ok(Type::u32),
                    IntType::u64 => Ok(Type::u64),
                    IntType::u128 => Ok(Type::u128),
                }
            }

            Expr::FloatLiteral(_, ty) => {
                // map FloatType to Type
                match ty {
                    FloatType::f32 => Ok(Type::f32),
                    FloatType::f64 => Ok(Type::f64),
                }
            }

            Expr::BoolLiteral(_) => Ok(Type::Bool),

            Expr::Variable(name) => self
                .lookup_var(name)
                .cloned()
                .ok_or_else(|| format!("Undeclared variable `{}`", name)),

            Expr::Assign { name, value } => {
                let value_type = self.type_check_expr(value)?;
                let var_type = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Assignment to undeclared variable `{}`", name))?;

                if &value_type != var_type {
                    return Err(format!(
                        "Type mismatch in assignment to `{}`: expected {:?}, found {:?}",
                        name, var_type, value_type
                    ));
                }
                Ok(value_type)
            }

            Expr::Binary {
                left,
                op: _,
                right,
                result_type: _,
            } => {
                let left_type = self.type_check_expr(left)?;
                let right_type = self.type_check_expr(right)?;

                if left_type != right_type {
                    return Err(format!(
                        "Type mismatch in binary operation: left is {:?}, right is {:?}",
                        left_type, right_type
                    ));
                }

                Ok(left_type)
            }

            Expr::Unary {
                op: _,
                expr,
                result_type: _,
            } => {
                let expr_type = self.type_check_expr(expr)?;
                // TODO: verify operator supports expr_type
                Ok(expr_type)
            }

            Expr::Call {
                name,
                args,
                return_type: _,
            } => {
                let (param_types, ret_type) = self
                    .lookup_fn(name)
                    .ok_or_else(|| format!("Undefined function `{}`", name))?
                    .clone();

                if param_types.len() != args.len() {
                    return Err(format!(
                        "Function `{}` expected {} arguments, got {}",
                        name,
                        param_types.len(),
                        args.len()
                    ));
                }

                for (arg_expr, expected_type) in args.iter().zip(param_types.iter()) {
                    let arg_type = self.type_check_expr(arg_expr)?;
                    if &arg_type != expected_type {
                        return Err(format!(
                            "Argument type mismatch in call to `{}`: expected {:?}, got {:?}",
                            name, expected_type, arg_type
                        ));
                    }
                }

                Ok(ret_type)
            }

            Expr::Cast { expr, target_type } => {
                // Check if cast is allowed
                let expr_type = self.type_check_expr(expr)?;

                Ok(target_type.clone())
            }

            Expr::Array(exprs, expr_elem_type) => {
                if exprs.is_empty() {
                    return Err("Cannot infer type of empty array".to_string());
                }

                for expr in exprs {
                    let ty = self.type_check_expr(expr)?;
                    if &ty != expr_elem_type {
                        return Err("Array elements must all have the same type".to_string());
                    }
                }

                Ok(Type::Array(Box::new(expr_elem_type.clone()), exprs.len()))
            }

            Expr::ArrayAccess {
                array,
                index,
                element_type,
            } => {
                let array_type = self.type_check_expr(array)?;
                let index_type = self.type_check_expr(index)?;

                if index_type != Type::i32 {
                    return Err("Array index must be of type i32".to_string());
                }

                match array_type {
                    Type::Array(inner_type, _) if *inner_type == *element_type => {
                        Ok(element_type.clone())
                    }
                    _ => Err("Attempted to index a non-array type or type mismatch".to_string()),
                }
            }
        }
    }
}
