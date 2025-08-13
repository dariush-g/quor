use crate::lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp};
use std::collections::{HashMap, HashSet};

pub struct TypeChecker {
    variables: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (Vec<Type>, Type)>,
    classes: Vec<String>,
    class_fields: HashMap<String, Vec<(String, Type)>>,
    current_return_type: Option<Type>,
    in_loop: bool,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self {
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
            classes: Vec::new(),
            class_fields: HashMap::new(),
            current_return_type: None,
            in_loop: false,
        }
    }
}

impl TypeChecker {
    pub fn analyze_program(program: Vec<Stmt>) -> Result<Vec<Stmt>, String> {
        let mut type_checker = TypeChecker::default();
        let mut checked_program = Vec::new();
        let mut function_names = HashSet::new();

        for stmt in &program {
            if let Stmt::FunDecl {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                if !function_names.insert(name) {
                    return Err(format!("Function '{name}' already declared"));
                }
                type_checker
                    .declare_fn(name, param_types, return_type.clone())
                    .map_err(|e| format!("Global scope error: {e}"))?;
            }
            if let Stmt::ClassDecl {
                name, instances, ..
            } = stmt
            {
                type_checker.classes.push(name.clone());
                type_checker
                    .class_fields
                    .insert(name.clone(), instances.clone());
            }
        }

        for stmt in program {
            let checked_stmt = type_checker.type_check_stmt(&stmt)?;
            checked_program.push(checked_stmt);
        }

        Ok(checked_program)
    }

    fn enter_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.variables.pop().expect("Cannot exit global scope");
    }

    fn declare_var(&mut self, name: &str, ty: Type) -> Result<(), String> {
        if self.variables.last().unwrap().contains_key(name) {
            return Err(format!("Variable '{name}' already declared in this scope"));
        }
        self.variables
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ty);
        Ok(())
    }

    fn lookup_var(&self, name: &str) -> Option<&Type> {
        for scope in self.variables.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn declare_fn(&mut self, name: &str, params: Vec<Type>, ret: Type) -> Result<(), String> {
        self.functions.insert(name.to_string(), (params, ret));
        Ok(())
    }

    fn lookup_fn(&self, name: &str) -> Option<&(Vec<Type>, Type)> {
        self.functions.get(name)
    }

    pub fn type_check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::BoolLiteral(_) => Ok(Type::Bool),
            Expr::IntLiteral(_) => Ok(Type::int),
            Expr::FloatLiteral(_) => Ok(Type::float),
            Expr::CharLiteral(_) => Ok(Type::Char),
            Expr::Variable(name) => self
                .lookup_var(name)
                .cloned()
                .ok_or_else(|| format!("Undeclared variable '{name}'")),
            Expr::Assign { name, value } => {
                let value_type = self.type_check_expr(value)?;
                let var_type = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Assignment to undeclared variable '{name}'"))?;

                if &value_type != var_type {
                    return Err(format!(
                        "Type mismatch in assignment to '{name}': expected {var_type:?}, found {value_type:?}"
                    ));
                }
                Ok(value_type)
            }
            Expr::Binary {
                left, op, right, ..
            } => {
                let left_type = self.type_check_expr(left)?;
                let right_type = self.type_check_expr(right)?;

                if left_type != right_type {
                    return Err(format!(
                        "Type mismatch in binary operation: left is {left_type:?}, right is {right_type:?}"
                    ));
                }

                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => {
                        if !matches!(left_type, Type::int | Type::float) {
                            return Err(format!(
                                "Arithmetic operations require numeric types, found {left_type:?}"
                            ));
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {}
                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => {
                        if !matches!(left_type, Type::int | Type::float) {
                            return Err(format!(
                                "Comparison operations require numeric types, found {left_type:?}"
                            ));
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != Type::Bool {
                            return Err("Logical operations require boolean operands".to_string());
                        }
                    }
                }

                match op {
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::And
                    | BinaryOp::Or => Ok(Type::Bool),
                    _ => Ok(left_type),
                }
            }
            Expr::Unary { op, expr, .. } => {
                let expr_type = self.type_check_expr(expr)?;

                match op {
                    UnaryOp::Not => {
                        if expr_type != Type::Bool {
                            return Err("Logical NOT requires a boolean operand".to_string());
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOp::Negate => {
                        if !matches!(expr_type, Type::int | Type::float) {
                            return Err("Negation requires a numeric operand".to_string());
                        }
                        Ok(expr_type)
                    }
                    UnaryOp::AddressOf => Ok(Type::Pointer(Box::new(expr_type))),
                    UnaryOp::Dereference => match expr_type {
                        Type::Pointer(inner) => Ok(*inner.clone()),
                        _ => Err("Cannot dereference a non-pointer type".to_string()),
                    },
                }
            }
            Expr::Call { name, args, .. } => {
                let (param_types, ret_type) = self
                    .lookup_fn(name)
                    .ok_or_else(|| format!("Undefined function '{name}'"))?
                    .clone();

                if param_types.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expected {} arguments, got {}",
                        name,
                        param_types.len(),
                        args.len()
                    ));
                }

                for (arg_expr, expected_type) in args.iter().zip(param_types.iter()) {
                    let arg_type = self.type_check_expr(arg_expr)?;
                    if &arg_type != expected_type {
                        return Err(format!(
                            "Argument type mismatch in call to '{name}': expected {expected_type:?}, got {arg_type:?}"
                        ));
                    }
                }

                Ok(ret_type)
            }
            Expr::Cast { expr, target_type } => {
                let expr_type = self.type_check_expr(expr)?;
                match (&expr_type, target_type) {
                    (Type::int, Type::float) | (Type::float, Type::int) => {}
                    (Type::Char, Type::int) | (Type::int, Type::Char) => {}
                    (from, to) if from == to => {}
                    _ => {
                        return Err(format!(
                            "Invalid cast from {expr_type:?} to {target_type:?}"
                        ));
                    }
                }
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

                Ok(Type::Array(
                    Box::new(expr_elem_type.clone()),
                    Some(exprs.len()),
                ))
            }
            Expr::ArrayAccess { array, index } => {
                let index_type = self.type_check_expr(index)?;

                if index_type != Type::int {
                    return Err("Array index must be of type int".to_string());
                }

                let arr = *array.clone();

                let name = match arr {
                    Expr::Variable(name) => name,
                    _ => return Err("Array type error".to_string()),
                };

                let array_full = self.type_check_expr(&Expr::Variable(name))?;

                match array_full {
                    Type::Array(ty, len) => {
                        let element_type = *ty;
                        if let Expr::IntLiteral(n) = **index {
                            if n >= len
                                .unwrap()
                                .try_into()
                                .expect("Error comparing index to array length")
                            {
                                return Err("Index out of bounds".to_string());
                            }
                        }
                        Ok(element_type)
                    }
                    _ => Err("Array type error".to_string()),
                }
            }
            Expr::AddressOf(expr) => Ok(Type::Pointer(Box::new(self.type_check_expr(expr)?))),
            Expr::DerefAssign { target, value } => {
                let target_type = self.type_check_expr(target)?;
                match target_type {
                    Type::Pointer(inner) => {
                        let val_ty = self.type_check_expr(value)?;
                        if val_ty != *inner {
                            return Err(format!(
                                "Type mismatch in deref assignment: expected {inner:?}, found {val_ty:?}"
                            ));
                        }
                        Ok(*inner)
                    }
                    _ => Err("Cannot assign through a non-pointer value".to_string()),
                }
            }
            Expr::ClassInit { name, params } => {
                let decl = self
                    .class_fields
                    .get(name)
                    .ok_or_else(|| format!("Unknown class '{name}'"))?
                    .clone();
                let mut decl_map: HashMap<&str, Type> = HashMap::new();
                let mut decl_order: Vec<(&str, Type)> = Vec::new();
                for (fname, fty) in &decl {
                    decl_map.insert(fname.as_str(), fty.clone());
                    decl_order.push((fname.as_str(), fty.clone()));
                }
                let mut seen = HashSet::new();
                for (fname, fexpr) in params {
                    if !seen.insert(fname) {
                        return Err(format!("Duplicate field initializer '{fname}'"));
                    }
                    let expected = decl_map
                        .get(fname.as_str())
                        .ok_or_else(|| format!("'{name}' has no field '{fname}'"))?;
                    let got = self.type_check_expr(fexpr)?;
                    if &got != expected {
                        return Err(format!(
                            "Type mismatch for field '{name}.{fname}': expected {expected:?}, got {got:?}"
                        ));
                    }
                }
                for (fname, _) in &decl {
                    if !seen.contains(fname) {
                        return Err(format!("Missing initializer for field '{name}.{fname}'"));
                    }
                }
                Ok(Type::Class(
                    decl_order.into_iter().map(|(_, t)| t).collect(),
                ))
            }
        }
    }

    pub fn type_check_stmt(&mut self, stmt: &Stmt) -> Result<Stmt, String> {
        match stmt {
            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => {
                let value_type = self.type_check_expr(value)?;
                let value_type = match value_type {
                    Type::Class(_) => Type::Class(Vec::new()),
                    _ => value_type,
                };

                if &value_type != var_type {
                    return Err(format!(
                        "Type mismatch in declaration of '{name}': expected {var_type:?}, found {value_type:?}"
                    ));
                }
                if let (Type::Array(_, decl_size), Expr::Array(elems, _)) = (var_type, value) {
                    let decl_size = decl_size.expect("Error with array length");
                    if !elems.is_empty() && elems.len() != decl_size {
                        return Err(format!(
                            "Array size mismatch in '{}': expected {}, found {}",
                            name,
                            decl_size,
                            elems.len()
                        ));
                    }
                }
                self.declare_var(name, var_type.clone())?;
                Ok(Stmt::VarDecl {
                    name: name.clone(),
                    var_type: var_type.clone(),
                    value: value.clone(),
                })
            }
            Stmt::FunDecl {
                name,
                params,
                return_type,
                body,
            } => {
                let param_types: Vec<Type> = params.iter().map(|(_, ty)| ty.clone()).collect();
                self.declare_fn(name, param_types.clone(), return_type.clone())?;
                self.enter_scope();
                self.current_return_type = Some(return_type.clone());
                for (param_name, param_type) in params {
                    self.declare_var(param_name, param_type.clone())?;
                }
                let mut checked_body = Vec::new();
                for stmt in body {
                    checked_body.push(self.type_check_stmt(stmt)?);
                }
                if *return_type != Type::Void {
                    let has_return = checked_body
                        .iter()
                        .any(|stmt| matches!(stmt, Stmt::Return(_)));
                    if !has_return {
                        return Err(format!(
                            "Function '{name}' with return type {return_type:?} is missing a return statement"
                        ));
                    }
                }
                self.exit_scope();
                self.current_return_type = None;
                Ok(Stmt::FunDecl {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: checked_body,
                })
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_type = self.type_check_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err("If condition must be boolean".to_string());
                }
                let checked_then = self.type_check_stmt(then_stmt)?;
                let checked_else = else_stmt
                    .as_ref()
                    .map(|stmt| self.type_check_stmt(stmt))
                    .transpose()?;
                Ok(Stmt::If {
                    condition: condition.clone(),
                    then_stmt: Box::new(checked_then),
                    else_stmt: checked_else.map(Box::new),
                })
            }
            Stmt::While { condition, body } => {
                let cond_type = self.type_check_expr(condition)?;
                if cond_type != Type::Bool {
                    return Err("While condition must be boolean".to_string());
                }
                let prev_in_loop = self.in_loop;
                self.in_loop = true;
                let checked_body = self.type_check_stmt(body)?;
                self.in_loop = prev_in_loop;
                Ok(Stmt::While {
                    condition: condition.clone(),
                    body: Box::new(checked_body),
                })
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.enter_scope();
                let prev_in_loop = self.in_loop;
                self.in_loop = true;

                let checked_init = init
                    .as_ref()
                    .map(|stmt| self.type_check_stmt(stmt))
                    .transpose()?;

                let checked_cond = condition
                    .as_ref()
                    .map(|expr| {
                        self.type_check_expr(expr)?;
                        Ok::<Expr, String>(expr.clone())
                    })
                    .transpose()?;

                let checked_update = update
                    .as_ref()
                    .map(|expr| {
                        self.type_check_expr(expr)?;
                        Ok::<Expr, String>(expr.clone())
                    })
                    .transpose()?;

                let checked_body = self.type_check_stmt(body)?;

                self.exit_scope();
                self.in_loop = prev_in_loop;

                Ok(Stmt::For {
                    init: checked_init.map(Box::new),
                    condition: checked_cond,
                    update: checked_update,
                    body: Box::new(checked_body),
                })
            }
            Stmt::Block(stmts) => {
                self.enter_scope();
                let mut checked_stmts = Vec::new();

                for stmt in stmts {
                    checked_stmts.push(self.type_check_stmt(stmt)?);
                }

                self.exit_scope();
                Ok(Stmt::Block(checked_stmts))
            }
            Stmt::Expression(expr) => {
                self.type_check_expr(expr)?;
                Ok(Stmt::Expression(expr.clone()))
            }
            Stmt::Return(expr) => {
                let return_type = match expr {
                    Some(expr) => self.type_check_expr(expr)?,
                    None => Type::Void,
                };

                match &self.current_return_type {
                    Some(expected) if *expected != return_type => {
                        return Err(format!(
                            "Return type mismatch: expected {expected:?}, found {return_type:?}"
                        ));
                    }
                    None => return Err("Return statement outside of function".to_string()),
                    _ => {}
                }

                Ok(Stmt::Return(expr.clone()))
            }
            Stmt::Break => {
                if !self.in_loop {
                    return Err("Break statement outside of loop".to_string());
                }
                Ok(Stmt::Break)
            }
            Stmt::Continue => {
                if !self.in_loop {
                    return Err("Continue statement outside of loop".to_string());
                }
                Ok(Stmt::Continue)
            }
            Stmt::ClassDecl {
                name,
                instances,
                funcs,
            } => {
                for (i, instance) in instances.clone().iter().enumerate() {
                    for (j, instance1) in instances.iter().enumerate() {
                        let n = instance.0.clone();
                        if n == instance1.0 && i != j {
                            return Err("Instances may not share a name".to_string());
                        }
                    }
                }

                for (i, stmt) in funcs.clone().iter().enumerate() {
                    match stmt {
                        Stmt::FunDecl { name, .. } => {
                            let name1 = name;
                            for (j, stmt1) in funcs.iter().enumerate() {
                                match stmt1 {
                                    Stmt::FunDecl { name, .. } => {
                                        if name1 == name && i != j {
                                            return Err(
                                                "Functions may not share a name".to_string()
                                            );
                                        }
                                    }
                                    _ => {
                                        return Err("Class functions must be functions".to_string());
                                    }
                                }
                            }
                        }
                        _ => {
                            return Err("Class functions must be functions".to_string());
                        }
                    }
                }

                Ok(Stmt::ClassDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    funcs: funcs.to_vec(),
                })
            }
        }
    }
}
