use crate::{
    frontend::{
        ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
        lexer::Lexer,
        parser::Parser,
    },
    midend::alias::AliasManager,
};

use std::{
    collections::{HashMap, HashSet},
    fs,
};

// TODO: Make a classes, globals, aliases, and class_fields for each ModuleId for namespace lookups

pub struct TypeChecker {
    variables: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (Vec<Type>, Type, Vec<String>)>,
    classes: HashMap<String, bool>,
    globals: HashMap<String, Type>,
    pub aliases: AliasManager,
    //                  struct name, instances
    class_fields: HashMap<String, Vec<(String, Type)>>,
    current_return_type: Option<Type>,
    in_loop: bool,
    called: Vec<String>,
    /// Number of modules (0..module_count); used to try qualified lookups for unqualified names.
    module_count: usize,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self {
            globals: HashMap::new(),
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
            classes: HashMap::new(),
            class_fields: HashMap::new(),
            current_return_type: None,
            in_loop: false,
            called: Vec::new(),
            aliases: AliasManager::default(),
            module_count: 1,
        }
    }
}

use std::path::{Path, PathBuf};

pub fn canonicalize_path(path: &str, base_dir: &Path) -> PathBuf {
    let path_buf = PathBuf::from(path);

    if path_buf.is_absolute() {
        path_buf.canonicalize().unwrap_or_else(|e| {
            eprintln!("Failed to canonicalize absolute path {path:?}: {e}");
            std::process::exit(1);
        })
    } else {
        let full_path = base_dir.join(path_buf);
        full_path.canonicalize().unwrap_or_else(|e| {
            eprintln!(
                "Failed to canonicalize relative path {path:?} against base {base_dir:?}: {e}"
            );
            std::process::exit(1);
        })
    }
}

impl TypeChecker {
    /// Expands imports, runs alias first+second pass so all names are qualified (`name.module_id`),
    /// and returns (flattened program, number of modules). Builtins must be registered for each module id in 0..module_count.
    pub fn process_program(&mut self, program: Vec<Stmt>, path_: &Path) -> (Vec<Stmt>, usize) {
        let entry_canonical = path_.canonicalize().unwrap_or_else(|e| {
            eprintln!("Failed to canonicalize entry path {path_:?}: {e}");
            std::process::exit(1);
        });
        let mut imported: HashSet<PathBuf> = HashSet::new();
        let files = self.collect_files(program, &entry_canonical, &mut imported);

        for (path, _) in &files {
            let canonical = path.to_string_lossy().into_owned();
            if !self.aliases.module_registry.contains_key(&canonical) {
                self.aliases.register_module(canonical);
            }
        }

        let module_count = self.aliases.module_registry.len();

        let mut first_passed: Vec<(PathBuf, Vec<Stmt>)> = files
            .into_iter()
            .map(|(path, stmts)| {
                let canonical = path.to_string_lossy().into_owned();
                let quor = self.aliases.get_module_mut(&canonical).unwrap();
                let new_stmts = quor.determine_symbols_stmts_first_pass(stmts);
                (path, new_stmts)
            })
            .collect();

        let mut result = Vec::new();
        for (path, stmts) in &mut first_passed {
            let canonical = path.to_string_lossy().into_owned();
            let quor = self.aliases.get_module(&canonical).unwrap();
            let stmts_to_use: Vec<Stmt> = if path == &entry_canonical {
                stmts
                    .iter()
                    .filter(|s| !matches!(s, Stmt::AtDecl(d, ..) if d.as_str() == "import"))
                    .cloned()
                    .collect()
            } else {
                stmts.clone()
            };
            let mut scope = HashSet::new();
            result.extend(quor.determine_symbols_stmts_second_pass(
                stmts_to_use,
                &mut scope,
                &self.aliases,
            ));
        }
        (result, module_count)
    }

    /// Collects (canonical_path, stmts) for the current file and all imports (depth-first).
    fn collect_files(
        &self,
        stmts: Vec<Stmt>,
        current_file: &Path,
        imported_files: &mut HashSet<PathBuf>,
    ) -> Vec<(PathBuf, Vec<Stmt>)> {
        let current_dir = current_file.parent().unwrap_or_else(|| Path::new("."));
        let mut result = Vec::new();

        for stmt in &stmts {
            if let Stmt::AtDecl(decl, param, _, _, _) = stmt {
                if decl.as_str() != "import" {
                    continue;
                }
                let mut path_param = param
                    .clone()
                    .unwrap_or_else(|| panic!("import without path"));
                let path_str = if path_param.ends_with('!') {
                    path_param.pop();
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    format!("{manifest_dir}/lib/{path_param}")
                } else {
                    path_param
                };
                let abs_path = canonicalize_path(&path_str, current_dir);
                if !imported_files.insert(abs_path.clone()) {
                    continue;
                }
                let source = match fs::read_to_string(&abs_path) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to read {abs_path:?}: {e}");
                        std::process::exit(1);
                    }
                };
                let tokens = match Lexer::new(source).tokenize() {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Lexer error: {e:?}");
                        std::process::exit(1);
                    }
                };
                let program_new = match Parser::new(tokens).parse() {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Parser error: {e:?}");
                        std::process::exit(1);
                    }
                };
                let mut sub = self.collect_files(program_new, &abs_path, imported_files);
                result.append(&mut sub);
            }
        }
        result.push((current_file.to_path_buf(), stmts));
        result
    }

    fn qualified_base_name(name: &str) -> &str {
        name.rsplit_once('.').map(|(base, _)| base).unwrap_or(name)
    }

    /// For special-case checks (sizeof, malloc): "new_string.0" -> "new_string", "s::new_string" -> "new_string".
    fn call_base_name(name: &str) -> &str {
        if let Some((_, after)) = name.split_once("::") {
            after
        } else {
            Self::qualified_base_name(name)
        }
    }

    pub fn analyze_program(
        mut program: Vec<Stmt>,
        path: &Path,
        module_count: usize,
    ) -> Result<Vec<Stmt>, String> {
        let mut type_checker = TypeChecker::default();
        type_checker.module_count = module_count;

        // Register builtins for every module so that e.g. string.qu (module 0) and test.qu (module 1) can both call malloc.
        for module_id in 0..module_count {
            let builtin = |name: &str| format!("{name}.{module_id}");

            for (i, stmt) in program.clone().iter().enumerate() {
                if let Stmt::AtDecl(decl, param, val, ..) = stmt {
                    if decl.as_str() == "const" || decl.as_str() == "const" {
                        let param = param
                            .clone()
                            .unwrap_or_else(|| panic!("Unable to locate define name"));

                        match val.clone().unwrap() {
                            Expr::IntLiteral(_) => {}
                            Expr::LongLiteral(_) => {}
                            Expr::FloatLiteral(_) => {}
                            Expr::BoolLiteral(_) => {}
                            Expr::StringLiteral(_) => {}
                            Expr::CharLiteral(_) => {}
                            Expr::StructInit { .. } => {}
                            Expr::Array(..) => {}
                            _ => {
                                panic!("Expected literal value for global definition")
                            }
                        }
                        type_checker.type_check_expr(&val.clone().unwrap())?;
                        let ty = val.clone().unwrap().get_type();

                        type_checker.globals.insert(param, ty);
                    }
                    if decl.as_str() == "union" {
                        if let Stmt::StructDecl {
                            name, instances, ..
                        } = program.get(i + 1).unwrap()
                        {
                            program[i + 1] = Stmt::StructDecl {
                                name: name.to_string(),
                                instances: instances.to_vec(),
                                union: true,
                            }
                        } else {
                            return Err("expected struct after union declaration".to_string());
                        }
                    }
                }
            }

            type_checker
                .declare_fn(&builtin("exit"), vec![Type::int], Type::Void, Vec::new())
                .map_err(|e| format!("Global scope error: {e}"))?;

            type_checker
                .declare_fn(
                    &builtin("free"),
                    vec![Type::Pointer(Box::new(Type::Void))],
                    Type::Void,
                    Vec::new(),
                )
                .map_err(|e| format!("Global scope error: {e}"))?;
            type_checker
                .declare_fn(
                    &builtin("malloc"),
                    vec![Type::int],
                    Type::Pointer(Box::new(Type::Void)),
                    Vec::new(),
                )
                .map_err(|e| format!("Global scope error: {e}"))?;
            type_checker
                .declare_fn(
                    &builtin("sizeof"),
                    vec![Type::Pointer(Box::new(Type::Void))],
                    Type::int,
                    Vec::new(),
                )
                .map_err(|e| format!("Global scope error: {e}"))?;

            type_checker
                .declare_fn(
                    &builtin("strlen"),
                    vec![Type::Pointer(Box::new(Type::Void))],
                    Type::int,
                    Vec::new(),
                )
                .map_err(|e| format!("Global scope error: {e}"))?;
        }

        // println!("{program:?}");

        // Register builtins

        // type_checker
        //     .declare_fn("print_bool", vec![Type::Bool], Type::Void)
        //     .map_err(|e| format!("Global scope error: {e}"))?;

        let mut checked_program = Vec::new();
        let mut function_names = HashSet::new();

        for stmt in program.iter_mut() {
            if let Stmt::FunDecl {
                name,
                params,
                return_type,
                attributes,
                ..
            } = stmt.clone()
            {
                let param_types = params.iter().map(|(_, ty)| ty.clone()).collect();
                if !function_names.insert(name.clone()) {
                    return Err("Function already declared".to_string());
                }
                type_checker
                    .declare_fn(name.as_str(), param_types, return_type.clone(), attributes)
                    .map_err(|e| format!("Global scope error: {e}"))?;
            }
            if let Stmt::StructDecl {
                name,
                instances,
                union,
                ..
            } = stmt
            {
                type_checker.classes.insert(name.clone(), *union);
                type_checker
                    .class_fields
                    .insert(name.clone(), instances.clone());
            }
        }

        for stmt in program {
            let checked_stmt = type_checker.type_check_stmt(&stmt)?;
            checked_program.push(checked_stmt);
        }

        for stmt in &mut checked_program {
            type_checker.fill_stmt_types(stmt);
        } // println!("{:?}", type_checker.variables);

        let mut remove_indices = Vec::new();
        for (i, stmt) in checked_program.iter().enumerate() {
            if let Stmt::FunDecl { name, params, .. } = stmt.clone()
                && !type_checker.called.contains(&name)
            {
                if name != "main" {
                    remove_indices.push(i);
                } else {
                    if !matches!(params.len(), 0 | 2) {
                        panic!(
                            "Warning: Function '{name}' may not contain such sequence of parameters."
                        );
                    }
                    if (params.len() == 2)
                        && params[1].1
                            != Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Char))))
                        || params[0].1 != Type::int
                    {
                        panic!(
                            "Warning: Function '{name}' parameters do not match expected signature (int, char**)."
                        );
                    }
                }
            }
        }

        for i in remove_indices.into_iter().rev() {
            checked_program.remove(i);
        }

        Ok(checked_program)
    }

    fn fill_stmt_types(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Expression(expr) => self.fill_expr_types(expr),
            Stmt::VarDecl { value, .. } => self.fill_expr_types(value),
            Stmt::Return(Some(expr)) => self.fill_expr_types(expr),
            Stmt::Return(None) => {}
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.fill_stmt_types(s);
                }
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.fill_expr_types(condition);
                self.fill_stmt_types(then_stmt);
                if let Some(else_stmt) = else_stmt {
                    self.fill_stmt_types(else_stmt);
                }
            }
            Stmt::While { condition, body } => {
                self.fill_expr_types(condition);
                self.fill_stmt_types(body);
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init) = init {
                    self.fill_stmt_types(init);
                }
                if let Some(cond) = condition {
                    self.fill_expr_types(cond);
                }
                if let Some(update) = update {
                    self.fill_expr_types(update);
                }
                self.fill_stmt_types(body);
            }
            Stmt::FunDecl { body, .. } => {
                for s in body {
                    self.fill_stmt_types(s);
                }
            }
            _ => {}
        }
    }

    fn fill_expr_types(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Variable(name, ty) => {
                if matches!(ty, Type::Unknown)
                    && let Some(var_type) = self.lookup_var(name)
                {
                    *ty = var_type.clone();
                }
            }
            Expr::Unary { expr: inner, .. } => self.fill_expr_types(inner),
            Expr::Binary { left, right, .. } => {
                self.fill_expr_types(left);
                self.fill_expr_types(right);
            }
            Expr::Assign { value, .. } => self.fill_expr_types(value),
            Expr::Call { name, args, .. } => {
                let base = TypeChecker::qualified_base_name(name);
                for arg in args {
                    if base == "sizeof" {
                        let param = &arg;
                        if let Expr::Variable(name, _) = param
                            && self.classes.contains_key(name)
                        {
                            self.variables.last_mut().unwrap().remove(name);
                            return;
                        }
                    }
                    self.fill_expr_types(arg);
                    self.called.push(name.to_string());
                    // println!("{name} {:?}", arg);
                }
            }
            Expr::ArrayAccess { array, index } => {
                self.fill_expr_types(array);
                self.fill_expr_types(index);
            }
            Expr::DerefAssign { target, value } => {
                self.fill_expr_types(target);
                self.fill_expr_types(value);
            }
            Expr::StructInit { params, .. } => {
                for (_, expr) in params {
                    self.fill_expr_types(expr);
                }
            }
            _ => {}
        }
    }

    fn enter_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    // fn exit_scope(&mut self) {
    //     self.variables.pop().expect("Cannot exit global scope");
    // }

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

        self.globals.get(name)
    }

    fn declare_fn(
        &mut self,
        name: &str,
        params: Vec<Type>,
        ret: Type,
        attrb: Vec<String>,
    ) -> Result<(), String> {
        self.functions
            .insert(name.to_string(), (params, ret, attrb));
        Ok(())
    }

    fn lookup_fn(&self, name: &str) -> Option<&(Vec<Type>, Type, Vec<String>)> {
        if let Some(f) = self.functions.get(name) {
            return Some(f);
        }
        // Namespace-qualified (e.g. s::new_string): resolve alias to module and try base_name.module_id
        if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            if let [namespace, base_name] = parts.as_slice() {
                for (_canonical, quor_file) in &self.aliases.module_registry {
                    if let Some(target_canonical) = quor_file.aliases.get(*namespace) {
                        if let Some(&module_id) = self.aliases.module_ids.get(target_canonical) {
                            let key = format!("{base_name}.{module_id}");
                            if let Some(f) = self.functions.get(&key) {
                                return Some(f);
                            }
                        }
                        break;
                    }
                }
            }
        }
        // Unqualified: try name.0, name.1, ... so builtins and cross-module calls resolve
        if !name.contains('.') {
            for id in 0..self.module_count {
                if let Some(f) = self.functions.get(&format!("{name}.{id}")) {
                    return Some(f);
                }
            }
        }
        None
    }

    /// Resolve a struct/class name (e.g. "string.1" when only "string.0" exists) by trying
    /// name, then name.0, name.1, ... so types from imported modules resolve.
    fn resolve_struct_name(&self, name: &str) -> Option<String> {
        if self.class_fields.contains_key(name) {
            return Some(name.to_string());
        }
        let base = Self::qualified_base_name(name);
        for id in 0..self.module_count {
            let key = format!("{base}.{id}");
            if self.class_fields.contains_key(&key) {
                return Some(key);
            }
        }
        None
    }

    pub fn type_check_expr(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::IndexAssign { value, .. } => Ok(value.get_type()),
            Expr::LongLiteral(_) => Ok(Type::Long),
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => {
                let ty = self
                    .type_check_expr(&Expr::InstanceVar(class_name.clone(), field.to_string()))?;

                if ty != self.type_check_expr(value)? {
                    return Err(format!("Error with class: {class_name}, field: {field}"));
                }

                Ok(ty)
            }
            Expr::StringLiteral(_) => Ok(Type::Pointer(Box::new(Type::Char))),
            Expr::BoolLiteral(_) => Ok(Type::Bool),
            Expr::IntLiteral(_) => Ok(Type::int),
            Expr::FloatLiteral(_) => Ok(Type::float),
            Expr::CharLiteral(_) => Ok(Type::Char),
            Expr::Variable(name, _) => self
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

                if let Type::Pointer(_) = left_type {
                    match op {
                        BinaryOp::Add | BinaryOp::Sub => {
                            if !matches!(right_type, Type::int) {
                                return Err(format!(
                                    "Arithmetic operations only for pointer types, found {right_type:?}"
                                ));
                            }

                            return Ok(left_type);
                        }
                        BinaryOp::Equal | BinaryOp::NotEqual => {
                            return Ok(Type::Bool);
                        }
                        _ => {}
                    }
                } else if let Type::Pointer(_) = right_type {
                    match op {
                        BinaryOp::Add | BinaryOp::Sub => {
                            if !matches!(left_type, Type::int) {
                                return Err(format!(
                                    "Arithmetic operations only for pointer types, found {left_type:?}"
                                ));
                            }
                            return Ok(right_type);
                        }
                        BinaryOp::Equal | BinaryOp::NotEqual => return Ok(Type::Bool),
                        _ => {}
                    }
                } else if left_type != right_type {
                    return Err(format!(
                        "Type mismatch in binary op: left is {left_type:?}, right is {right_type:?}"
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
                        if !matches!(left_type, Type::int | Type::float | Type::Char) {
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
                    // TODO: ADD BITWISE
                    _ => {}
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
                        Type::Void => Ok(Type::Void),
                        _ => Err("Cannot dereference a non-pointer type".to_string()),
                    },
                }
            }
            Expr::Call { name, args, .. } => {
                let base = Self::call_base_name(name);

                if base == "sizeof" {
                    let param = &args[0];
                    if let Expr::Variable(name1, _) = param
                        && self.classes.contains_key(name1)
                    {
                        self.variables.last_mut().unwrap().remove(name);
                        return Ok(Type::int);
                    }
                }

                let (param_types, ret_type, attributes) = self
                    .lookup_fn(name)
                    .ok_or_else(|| {
                        let name = Self::call_base_name(name);
                        format!("Undefined function '{name}'")
                    })?
                    .clone();

                if param_types.len() != args.len()
                    && !attributes.contains(&"any_params".to_string())
                {
                    return Err(format!(
                        "Function '{}' expected {} arguments, got {}",
                        name,
                        param_types.len(),
                        args.len()
                    ));
                }

                for (arg_expr, expected_type) in args.iter().zip(param_types.iter()) {
                    let arg_type = self.type_check_expr(arg_expr)?;

                    let arg_type = base_type(&arg_type);

                    if matches!(expected_type, Type::Pointer(inner) if **inner == Type::Void)
                        && matches!(arg_type, Type::Pointer(_))
                    {
                        continue; // allow any pointer as void*
                    }

                    if base == "sizeof" {
                        return Ok(Type::int);
                    }

                    if arg_type != *expected_type {
                        if base == "malloc"
                            && let Type::StructLiteral(_) = arg_type
                        {
                            return Ok(Type::Pointer(Box::new(Type::Void)));
                        }
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
                    (Type::Pointer(void), Type::Pointer(_)) if **void == Type::Void => {}
                    (Type::int, Type::float) | (Type::float, Type::int) => {}
                    (Type::Char, Type::int) | (Type::int, Type::Char) => {}
                    (Type::Void, Type::int)
                    | (Type::Void, Type::Bool)
                    | (Type::Void, Type::Char)
                    | (Type::Void, Type::float) => {}
                    (Type::Void, Type::Struct { .. }) => {}
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
                    Expr::Variable(name, _) => name,
                    _ => return Err("Array type error".to_string()),
                };

                let array_full = self.type_check_expr(&Expr::Variable(name, Type::Unknown))?;

                match array_full {
                    Type::Array(ty, len) => {
                        let element_type = *ty;
                        if let Expr::IntLiteral(n) = **index
                            && len.is_some()
                            && n >= len
                                .unwrap()
                                .try_into()
                                .expect("Error comparing index to array length")
                        {
                            return Err("Index out of bounds".to_string());
                        }
                        Ok(element_type)
                    }
                    Type::Pointer(ty) => {
                        let element_type = *ty;
                        if let Expr::IntLiteral(_) = **index {
                            return Ok(element_type);
                        }

                        Err("Array type error".to_string())
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
                        if val_ty != *inner && *inner != Type::Void {
                            return Err(format!(
                                "Type mismatch in deref assignment: expected {inner:?}, found {val_ty:?}"
                            ));
                        }

                        if *inner == Type::Void {
                            let val_ty = self.type_check_expr(value)?;

                            // Update the variable type in-place
                            if let Expr::Variable(n, _) = *target.clone()
                                && let Some(var_ty) = self.variables.last_mut().unwrap().get_mut(&n)
                            {
                                *var_ty = Type::Pointer(Box::new(val_ty.clone()));
                            }

                            return Ok(val_ty);
                        }

                        Ok(*inner)
                    }
                    _ => Err("Cannot assign through a non-pointer value".to_string()),
                }
            }
            Expr::StructInit { name, params } => {
                let _base = Self::qualified_base_name(name);

                let class_fields = self
                    .class_fields
                    .get(name)
                    .ok_or_else(|| format!("Undefined class: '{name}'"))?
                    .clone();

                if *self.classes.get(name).unwrap() {
                    if self.class_fields.get(name).unwrap().is_empty() {
                        // TODO: add warning here for making a union with no fields -> pointless
                        return Ok(Type::Struct {
                            name: name.clone(),
                            instances: vec![],
                        });
                    }
                    if params.len() != 1 {
                        return Err("Expected one parameter for union init".to_string());
                    }

                    let found = class_fields.iter().find_map(|field| {
                        if field.0 == params[0].0 && field.1 == params[0].1.get_type() {
                            Some(Ok(Type::Struct {
                                name: name.clone(),
                                instances: vec![(params[0].0.clone(), params[0].1.get_type())],
                            }))
                        } else {
                            None
                        }
                    });

                    if let Some(result) = found {
                        return result;
                    } else {
                        return Err(format!(
                            "Unexpected parameter for union init: {:?}",
                            params.first()
                        ));
                    }
                }

                let mut decl_map: HashMap<&str, Type> = HashMap::new();
                let mut decl_order: Vec<(&str, Type)> = Vec::new();

                for (fname, fty) in &class_fields {
                    decl_map.insert(fname.as_str(), fty.clone());
                    decl_order.push((fname.as_str(), fty.clone()));
                }

                let mut seen = HashSet::new();
                for (fname, fexpr) in params {
                    if !seen.insert(fname.clone()) {
                        return Err(format!("Duplicate field initializer '{fname}'"));
                    }
                    let expected = decl_map
                        .get(fname.as_str())
                        .ok_or_else(|| format!("Struct '{name}' has no field '{fname}'"))?;
                    let got = self.type_check_expr(fexpr)?;

                    let got = if let Type::Array(ty, len) = got {
                        // println!("{len:?}");
                        Type::Array(ty.clone(), len)
                    } else {
                        got.clone()
                    };

                    let got = match got {
                        Type::Struct { name, .. } => Type::Struct {
                            name,
                            instances: Vec::new(),
                        },
                        _ => got,
                    };

                    if let Type::Struct { .. } = base_type(&got) {
                    } else {
                        match got {
                            Type::Array(_, _) => {}
                            _ => {
                                let ty1 = base_type(&got);
                                let ty2 = base_type(expected);

                                // Only compare struct names, not their fields
                                match (&ty1, &ty2) {
                                    (
                                        Type::Struct { name: n1, .. },
                                        Type::Struct { name: n2, .. },
                                    ) if n1 == n2 => {}
                                    _ if &got == expected => {}
                                    _ => {
                                        return Err(format!(
                                            "Type mismatch for field '{fname}': expected {expected:?}, got {got:?}"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                // Check that all required fields are initialized
                for (fname, _) in &class_fields {
                    if !seen.contains(fname) {
                        return Err(format!("Missing initializer for field '{name}.{fname}'"));
                    }
                }

                Ok(Type::Struct {
                    name: name.clone(),
                    instances: decl_order
                        .iter()
                        .map(|(name, ty)| (name.to_string(), ty.clone()))
                        .collect(),
                })
            }
            // Expr::InstanceVar(class_name, instance_name) => {
            //     let ty = self
            //         .lookup_var(class_name)
            //         .ok_or_else(|| format!("Unknown variable: '{class_name}'"))?;

            //     match ty {
            //         Type::Pointer(ty) => match *ty.clone() {
            //             Type::Struct { name, .. } => {
            //                 let fields = self
            //                     .class_fields
            //                     .get(&name)
            //                     .unwrap_or_else(|| panic!("Couldn't find class: '{name}'"));

            //                 // Find the field in the class instances
            //                 for (field_name, field_type) in fields {
            //                     if field_name == instance_name {
            //                         return Ok(field_type.clone());
            //                     }
            //                 }
            //                 Err(format!(
            //                     "Unknown field '{instance_name}' in class '{class_name}'"
            //                 ))
            //             }
            //             _ => Err(format!("'{class_name}' is not a class instance")),
            //         },
            //         Type::Struct { name, .. } => {
            //             let fields = self
            //                 .class_fields
            //                 .get(name)
            //                 .unwrap_or_else(|| panic!("Couldn't find class: '{name}'"));

            //             // Find the field in the class instances
            //             for (field_name, field_type) in fields {
            //                 if field_name == instance_name {
            //                     return Ok(field_type.clone());
            //                 }
            //             }
            //             Err(format!(
            //                 "Unknown field '{instance_name}' in class '{class_name}'"
            //             ))
            //         }
            //         _ => Err(format!("'{class_name}' is not a class instance")),
            //     }
            // } // Expr::StructLiteral(name) => Ok(Type::Struct {
            //     name: name.to_string(),
            //     instances: Vec::new(),
            // }),
            Expr::InstanceVar(class_name, instance_name) => {
                let ty = self
                    .lookup_var(class_name)
                    .ok_or_else(|| format!("Unknown variable: '{class_name}'"))?;

                let mut base = base_type(ty);

                while let Type::Pointer(ref inside) = base {
                    base = base_type(inside);
                }

                match base {
                    Type::Pointer(ty) => match *ty.clone() {
                        Type::Struct { name, .. } => {
                            let fields = self
                                .class_fields
                                .get(&name)
                                .unwrap_or_else(|| panic!("Couldn't find class: '{name}'"));
                            fields
                                .iter()
                                .find(|(field_name, _)| field_name == instance_name)
                                .map(|(_, ty)| ty.clone())
                                .ok_or_else(|| {
                                    format!(
                                        "Unknown field '{instance_name}' in class '{class_name}'"
                                    )
                                })
                        }
                        _ => Err(format!("'{class_name}' is not a class instance")),
                    },
                    Type::Struct { name, .. } => {
                        // Add direct class instance access
                        let fields = self
                            .class_fields
                            .get(&name.clone())
                            .unwrap_or_else(|| panic!("Couldn't find class: '{name}'"));
                        fields
                            .iter()
                            .find(|(field_name, _)| field_name == instance_name)
                            .map(|(_, ty)| ty.clone())
                            .ok_or_else(|| {
                                format!("Unknown field '{instance_name}' in class '{class_name}'")
                            })
                    }
                    _ => Err(format!("'{class_name}' is not a class instance")),
                }
            }
            Expr::CompoundAssign { name, op: _, value } => {
                let var_type = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Unknown variable '{name}'"))?
                    .clone();

                let value_type = self.type_check_expr(value)?;
                if var_type != value_type {
                    return Err(format!(
                        "Type mismatch in compound assignment: variable '{}' is {:?}, but value is {:?}",
                        name, var_type, value_type
                    ));
                }

                Ok(var_type)
            }
            Expr::PreIncrement { name } | Expr::PostIncrement { name } => {
                let var_type = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Unknown variable '{name}'"))?;

                match var_type {
                    Type::int | Type::Long => Ok(var_type.clone()),
                    _ => Err(format!(
                        "Cannot increment non-numeric variable '{name}' of type {:?}",
                        var_type
                    )),
                }
            }
            Expr::PreDecrement { name } | Expr::PostDecrement { name } => {
                let var_type = self
                    .lookup_var(name)
                    .ok_or_else(|| format!("Unknown variable '{name}'"))?;

                match var_type {
                    Type::int | Type::Long => Ok(var_type.clone()),
                    _ => Err(format!(
                        "Cannot decrement non-numeric variable '{name}' of type {:?}",
                        var_type
                    )),
                }
            }
        }
    }

    pub fn type_check_stmt(&mut self, stmt: &Stmt) -> Result<Stmt, String> {
        match stmt {
            Stmt::AtDecl(decl, ..) => match decl.as_str() {
                "import" => Ok(stmt.clone()),
                "const" | "CONST" => Ok(stmt.clone()),
                "union" => Ok(stmt.clone()),
                "keep_asm" => Ok(stmt.clone()),
                "trust_ret" => Ok(stmt.clone()),
                "__asm__" | "asm" | "_asm_" => Ok(stmt.clone()),
                "__asm_bss__" | "_asm_bss_" | "asm_bss" => Ok(stmt.clone()),
                "__asm_ro__" | "_asm_ro_" | "asm_ro" => Ok(stmt.clone()),
                "any_params" | "ANY_PARAMS" => Ok(stmt.clone()),
                // "public" => Ok(stmt.clone()),
                // "private" => Ok(stmt.clone()),
                _ => Err(format!("Unknown @ declaration: '{decl}'")),
            },

            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => {
                let resolved_type = if let Type::Struct {
                    name: class_name,
                    instances,
                } = &var_type
                {
                    if instances.is_empty() {
                        let resolved = self
                            .resolve_struct_name(class_name)
                            .ok_or_else(|| format!("Undefined class: '{class_name}'"))?;
                        let class_fields = self
                            .class_fields
                            .get(&resolved)
                            .ok_or_else(|| format!("Undefined class: '{class_name}'"))?;
                        Type::Struct {
                            name: resolved,
                            instances: class_fields.clone(),
                        }
                    } else {
                        var_type.clone()
                    }
                } else {
                    var_type.clone()
                };

                let value_type = self.type_check_expr(value)?;

                // println!("{value_type:?}, {resolved_type:?}");

                if value_type != resolved_type {
                    if let Type::Array(ty1, _) = value_type.clone() {
                        match resolved_type.clone() {
                            Type::Array(ty, _) => {
                                if ty1 != ty {
                                    return Err(format!(
                                        "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                                    ));
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                                ));
                            }
                        }
                    } else if let Type::Pointer(boxed_expected) = base_type(&resolved_type) {
                        if let Type::Pointer(boxed_value) = base_type(&value_type) {
                            match (&*boxed_expected, &*boxed_value) {
                                (
                                    Type::Struct {
                                        name: expected_name,
                                        ..
                                    },
                                    Type::Struct {
                                        name: value_name, ..
                                    },
                                ) => {
                                    if expected_name != value_name {
                                        return Err(format!(
                                            "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(format!(
                                        "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                                    ));
                                }
                            }
                        } else {
                            return Err(format!(
                                "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                            ));
                        }
                    } else if let Type::Struct { name: name2, .. } = base_type(&resolved_type) {
                        if let Type::Struct { name: name1, .. } = base_type(&value_type) {
                            if name1 != name2 {
                                return Err(format!(
                                    "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Type mismatch in declaration of '{name}': expected {resolved_type:?}, found {value_type:?}"
                        ));
                    }
                }

                if let (Type::Array(_, decl_size), Expr::Array(elems, _)) = (var_type, value) {
                    let decl_size = decl_size.expect("Error with array length");
                    if !elems.is_empty() && elems.len() > decl_size {
                        return Err(format!(
                            "Array size mismatch in '{}': expected {}, found {}",
                            name,
                            decl_size,
                            elems.len()
                        ));
                    }
                }

                self.declare_var(name, resolved_type.clone())?;
                Ok(Stmt::VarDecl {
                    name: name.clone(),
                    var_type: resolved_type,
                    value: value.clone(),
                })
            }
            Stmt::FunDecl {
                name,
                params,
                return_type,
                body,
                attributes,
            } => {
                let param_types: Vec<Type> = params.iter().map(|(_, ty)| ty.clone()).collect();
                self.declare_fn(
                    name,
                    param_types.clone(),
                    return_type.clone(),
                    attributes.to_vec(),
                )?;
                self.enter_scope();
                self.current_return_type = Some(return_type.clone());
                for (param_name, param_type) in params {
                    self.declare_var(param_name, param_type.clone())?;
                }
                let mut checked_body = Vec::new();
                for stmt in body {
                    checked_body.push(self.type_check_stmt(stmt)?);
                }

                // Check if function has @trust_ret attribute
                let has_trust_ret = attributes.contains(&"trust_ret".to_string());
                // println!("Function '{}' attributes: {:?}, has_trust_ret: {}", name, attributes, has_trust_ret);

                if *return_type != Type::Void && !has_trust_ret {
                    let has_return = checked_body
                        .iter()
                        .any(|stmt| matches!(stmt, Stmt::Return(_)));
                    if !has_return {
                        return Err(format!(
                            "Function '{name}' with return type {return_type:?} is missing a return statement"
                        ));
                    }
                }
                // self.exit_scope();
                self.current_return_type = None;
                Ok(Stmt::FunDecl {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: checked_body,
                    attributes: attributes.clone(),
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

                // self.exit_scope();
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

                // self.exit_scope();
                Ok(Stmt::Block(checked_stmts))
            }
            Stmt::Expression(expr) => {
                self.type_check_expr(expr)?;
                Ok(Stmt::Expression(expr.clone()))
            }
            Stmt::Return(expr) => {
                let mut return_type = match expr {
                    Some(expr) => self.type_check_expr(expr)?,
                    None => Type::Void,
                };

                return_type = base_type(&return_type);

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
            Stmt::StructDecl {
                name,
                instances,
                union,
            } => {
                for (i, instance) in instances.clone().iter().enumerate() {
                    for (j, instance1) in instances.iter().enumerate() {
                        let n = instance.0.clone();
                        if n == instance1.0 && i != j {
                            return Err("Instances may not share a name".to_string());
                        }
                    }
                }

                Ok(Stmt::StructDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    union: *union,
                })
            }
        }
    }
}

pub fn base_type(ty: &Type) -> Type {
    match ty {
        Type::Array(ty, ..) => Type::Array(ty.clone(), None),
        Type::Struct { name, .. } => Type::Struct {
            name: name.clone(),
            instances: Vec::new(),
        },
        Type::Pointer(inside) => Type::Pointer(Box::new(base_type(inside))),
        _ => ty.clone(),
    }
}
