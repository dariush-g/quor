use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::frontend::ast::*;

// Resolves an import path relative to the current file into a canonical path string.
pub fn resolve_import_path(import_path: &str, current_file: &str) -> CanonicalFile {
    let path = if import_path.ends_with('!') {
        let mut p = import_path.to_string();
        p.pop();
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(format!("{manifest_dir}/lib/{p}"))
    } else {
        PathBuf::from(import_path)
    };
    let base_dir = Path::new(current_file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let full = if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    };
    full.canonicalize()
        .unwrap_or_else(|e| {
            eprintln!(
                "Failed to resolve import path {:?} from {current_file:?}: {e}",
                full
            );
            std::process::exit(1);
        })
        .to_string_lossy()
        .into_owned()
}

#[derive(Clone, Debug, Default)]
pub struct ModuleSymbols {
    // Struct qualified name (e.g. "Point.0") -> is_union
    pub structs: HashMap<String, bool>,
    // Struct qualified name -> field names and types
    pub struct_fields: HashMap<String, Vec<(String, Type)>>,
    // Function qualified name -> (param types, return type, attributes)
    pub functions: HashMap<String, (Vec<Type>, Type, Vec<String>)>,
    // Global qualified name -> type
    pub globals: HashMap<String, Type>,
}

impl ModuleSymbols {
    // Look up a function by qualified name (e.g. "sin.3") for type analysis.
    pub fn lookup_fn(&self, qualified_name: &str) -> Option<&(Vec<Type>, Type, Vec<String>)> {
        self.functions.get(qualified_name)
    }

    // Look up a struct by qualified name.
    pub fn lookup_struct(&self, qualified_name: &str) -> Option<&bool> {
        self.structs.get(qualified_name)
    }

    // Look up struct fields by qualified name.
    pub fn lookup_struct_fields(&self, qualified_name: &str) -> Option<&Vec<(String, Type)>> {
        self.struct_fields.get(qualified_name)
    }

    // Look up a global by qualified name.
    pub fn lookup_global(&self, qualified_name: &str) -> Option<&Type> {
        self.globals.get(qualified_name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct QuorFile {
    pub file: CanonicalFile,
    pub module_id: usize,
    pub symbols: ModuleSymbols,
    pub aliases: HashMap<Alias, CanonicalFile>,
}

/// Qualify struct names in a type with the given module_id so type checker lookups work.
fn qualify_type(ty: Type, module_id: usize) -> Type {
    match ty {
        Type::Struct {
            name,
            instances,
        } => Type::Struct {
            name: format!("{name}.{module_id}"),
            instances: instances
                .into_iter()
                .map(|(f, t)| (f, qualify_type(t, module_id)))
                .collect(),
        },
        Type::StructLiteral(name) => Type::StructLiteral(format!("{name}.{module_id}")),
        Type::Array(inner, len) => Type::Array(Box::new(qualify_type(*inner, module_id)), len),
        Type::Pointer(inner) => Type::Pointer(Box::new(qualify_type(*inner, module_id))),
        other => other,
    }
}

impl QuorFile {
    pub fn determine_symbols_stmts_first_pass(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut named_stmts: Vec<Stmt> = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::AtDecl(decl, name_opt, val, content, alias_opt) => {
                    if decl.as_str() == "const" {
                        let name = name_opt
                            .clone()
                            .unwrap_or_else(|| panic!("const without name"));
                        let qualified = format!("{name}.{}", self.module_id);
                        let ty = val
                            .as_ref()
                            .map_or(Type::Unknown, |e| qualify_type(e.get_type(), self.module_id));
                        self.symbols.globals.insert(qualified.clone(), ty);
                        named_stmts.push(Stmt::AtDecl(
                            decl,
                            Some(qualified),
                            val,
                            content,
                            alias_opt,
                        ));
                    } else if decl.as_str() == "import" {
                        let path = name_opt
                            .as_ref()
                            .unwrap_or_else(|| panic!("import without path"));
                        let canonical = resolve_import_path(path, &self.file);
                        let alias = alias_opt.unwrap_or_else(|| {
                            Path::new(&canonical)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("mod")
                                .to_string()
                        });
                        self.aliases.insert(alias.clone(), canonical);
                        named_stmts.push(Stmt::AtDecl(decl, name_opt, val, content, Some(alias)));
                    } else {
                        named_stmts.push(Stmt::AtDecl(decl, name_opt, val, content, alias_opt));
                    }
                }
                Stmt::FunDecl {
                    name,
                    params,
                    return_type,
                    body,
                    attributes,
                } => {
                    let name = format!("{name}.{}", self.module_id);
                    let param_types: Vec<Type> = params
                        .iter()
                        .map(|(_, ty)| qualify_type(ty.clone(), self.module_id))
                        .collect();
                    let return_type = qualify_type(return_type, self.module_id);
                    self.symbols.functions.insert(
                        name.clone(),
                        (param_types.clone(), return_type.clone(), attributes.clone()),
                    );
                    let params = params
                        .into_iter()
                        .map(|(n, ty)| (n, qualify_type(ty, self.module_id)))
                        .collect();
                    named_stmts.push(Stmt::FunDecl {
                        name,
                        params,
                        return_type,
                        body: self.determine_symbols_stmts_first_pass(body),
                        attributes,
                    })
                }
                Stmt::StructDecl {
                    name,
                    instances,
                    union,
                } => {
                    let name = format!("{name}.{}", self.module_id);
                    let instances: Vec<(String, Type)> = instances
                        .into_iter()
                        .map(|(f, ty)| (f, qualify_type(ty, self.module_id)))
                        .collect();
                    self.symbols.structs.insert(name.clone(), union);
                    self.symbols.struct_fields.insert(name.clone(), instances.clone());

                    named_stmts.push(Stmt::StructDecl {
                        name,
                        instances,
                        union,
                    })
                }
                Stmt::If {
                    condition,
                    then_stmt,
                    else_stmt,
                } => {
                    let then_stmt = Box::new(match *then_stmt {
                        Stmt::Block(stmts) => {
                            Stmt::Block(self.determine_symbols_stmts_first_pass(stmts))
                        }
                        other => other,
                    });
                    let else_stmt = else_stmt.map(|stmt| {
                        Box::new(match *stmt {
                            Stmt::Block(stmts) => {
                                Stmt::Block(self.determine_symbols_stmts_first_pass(stmts))
                            }
                            other => other,
                        })
                    });
                    named_stmts.push(Stmt::If {
                        condition,
                        then_stmt,
                        else_stmt,
                    });
                }
                Stmt::While { condition, body } => {
                    let body = Box::new(match *body {
                        Stmt::Block(stmts) => {
                            Stmt::Block(self.determine_symbols_stmts_first_pass(stmts))
                        }
                        other => other,
                    });
                    named_stmts.push(Stmt::While { condition, body });
                }
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => {
                    let init = init.map(|s| {
                        Box::new(match *s {
                            Stmt::Block(stmts) => {
                                Stmt::Block(self.determine_symbols_stmts_first_pass(stmts))
                            }
                            other => other,
                        })
                    });
                    let body = Box::new(match *body {
                        Stmt::Block(stmts) => {
                            Stmt::Block(self.determine_symbols_stmts_first_pass(stmts))
                        }
                        other => other,
                    });
                    named_stmts.push(Stmt::For {
                        init,
                        condition,
                        update,
                        body,
                    });
                }
                Stmt::VarDecl {
                    name,
                    var_type,
                    value,
                } => {
                    named_stmts.push(Stmt::VarDecl {
                        name,
                        var_type: qualify_type(var_type, self.module_id),
                        value,
                    });
                }
                Stmt::Block(stmts) => {
                    named_stmts.push(Stmt::Block(self.determine_symbols_stmts_first_pass(stmts)));
                }
                _ => named_stmts.push(stmt),
            }
        }

        named_stmts
    }

    /// Resolve a name (possibly "namespace::name") to the qualified form "{name}.{module_id}".
    /// If the name has no "::" and is in `locals`, it is left unchanged (local variable).
    pub fn resolve_name(
        &self,
        name: &str,
        alias_manager: &AliasManager,
        locals: &HashSet<String>,
    ) -> String {
        if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            let (namespace, base_name) = match parts.as_slice() {
                [ns, base] => (*ns, *base),
                _ => return name.to_string(),
            };
            let canonical = match self.aliases.get(namespace) {
                Some(c) => c.clone(),
                None => return name.to_string(),
            };
            let module_id = match alias_manager.module_ids.get(&canonical) {
                Some(&id) => id,
                None => return name.to_string(),
            };
            format!("{base_name}.{module_id}")
        } else if locals.contains(name) {
            name.to_string()
        } else {
            format!("{name}.{}", self.module_id)
        }
    }

    /// using current file's aliases and module_id. `locals` is the set of local variable names in scope.
    pub fn determine_symbols_expr(
        &self,
        expr: Expr,
        alias_manager: &AliasManager,
        locals: &HashSet<String>,
    ) -> Expr {
        match expr {
            Expr::StructInit { name, params } => {
                let name = self.resolve_name(&name, alias_manager, locals);
                let params = params
                    .into_iter()
                    .map(|(f, e)| (f, self.determine_symbols_expr(e, alias_manager, locals)))
                    .collect();
                Expr::StructInit { name, params }
            }
            Expr::AddressOf(expr) => Expr::AddressOf(Box::new(self.determine_symbols_expr(
                *expr,
                alias_manager,
                locals,
            ))),
            Expr::DerefAssign { target, value } => Expr::DerefAssign {
                target: Box::new(self.determine_symbols_expr(*target, alias_manager, locals)),
                value: Box::new(self.determine_symbols_expr(*value, alias_manager, locals)),
            },
            Expr::InstanceVar(struct_name, field) => Expr::InstanceVar(
                self.resolve_name(&struct_name, alias_manager, locals),
                field,
            ),
            Expr::Variable(name, ty) => {
                Expr::Variable(self.resolve_name(&name, alias_manager, locals), ty)
            }
            Expr::Assign { name, value } => Expr::Assign {
                name: self.resolve_name(&name, alias_manager, locals),
                value: Box::new(self.determine_symbols_expr(*value, alias_manager, locals)),
            },
            Expr::CompoundAssign { name, op, value } => Expr::CompoundAssign {
                name: self.resolve_name(&name, alias_manager, locals),
                op,
                value: Box::new(self.determine_symbols_expr(*value, alias_manager, locals)),
            },
            Expr::PreIncrement { name } => Expr::PreIncrement {
                name: self.resolve_name(&name, alias_manager, locals),
            },
            Expr::PostIncrement { name } => Expr::PostIncrement {
                name: self.resolve_name(&name, alias_manager, locals),
            },
            Expr::PreDecrement { name } => Expr::PreDecrement {
                name: self.resolve_name(&name, alias_manager, locals),
            },
            Expr::PostDecrement { name } => Expr::PostDecrement {
                name: self.resolve_name(&name, alias_manager, locals),
            },
            Expr::Binary {
                left,
                op,
                right,
                result_type,
            } => Expr::Binary {
                left: Box::new(self.determine_symbols_expr(*left, alias_manager, locals)),
                op,
                right: Box::new(self.determine_symbols_expr(*right, alias_manager, locals)),
                result_type,
            },
            Expr::Unary {
                op,
                expr,
                result_type,
            } => Expr::Unary {
                op,
                expr: Box::new(self.determine_symbols_expr(*expr, alias_manager, locals)),
                result_type,
            },
            Expr::Call {
                name,
                args,
                return_type,
            } => Expr::Call {
                name: self.resolve_name(&name, alias_manager, locals),
                args: args
                    .into_iter()
                    .map(|e| self.determine_symbols_expr(e, alias_manager, locals))
                    .collect(),
                return_type,
            },
            Expr::Cast { expr, target_type } => Expr::Cast {
                expr: Box::new(self.determine_symbols_expr(*expr, alias_manager, locals)),
                target_type,
            },
            Expr::Array(exprs, ty) => Expr::Array(
                exprs
                    .into_iter()
                    .map(|e| self.determine_symbols_expr(e, alias_manager, locals))
                    .collect(),
                ty,
            ),
            Expr::ArrayAccess { array, index } => Expr::ArrayAccess {
                array: Box::new(self.determine_symbols_expr(*array, alias_manager, locals)),
                index: Box::new(self.determine_symbols_expr(*index, alias_manager, locals)),
            },
            Expr::IndexAssign {
                array,
                index,
                value,
            } => Expr::IndexAssign {
                array: Box::new(self.determine_symbols_expr(*array, alias_manager, locals)),
                index: Box::new(self.determine_symbols_expr(*index, alias_manager, locals)),
                value: Box::new(self.determine_symbols_expr(*value, alias_manager, locals)),
            },
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => Expr::FieldAssign {
                class_name: self.resolve_name(&class_name, alias_manager, locals),
                field,
                value: Box::new(self.determine_symbols_expr(*value, alias_manager, locals)),
            },
            other => other,
        }
    }

    /// Second pass: rewrite all expression references in stmts to qualified names.
    /// Maintains a scope of local variable names so locals are not qualified.
    pub fn determine_symbols_stmts_second_pass(
        &self,
        stmts: Vec<Stmt>,
        scope: &mut HashSet<String>,
        alias_manager: &AliasManager,
    ) -> Vec<Stmt> {
        let mut out = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::VarDecl {
                    name,
                    var_type,
                    value,
                } => {
                    scope.insert(name.clone());
                    out.push(Stmt::VarDecl {
                        name: name.clone(),
                        var_type,
                        value: self.determine_symbols_expr(value, alias_manager, scope),
                    });
                }
                Stmt::Expression(expr) => {
                    out.push(Stmt::Expression(self.determine_symbols_expr(
                        expr,
                        alias_manager,
                        scope,
                    )));
                }
                Stmt::Return(opt) => {
                    out.push(Stmt::Return(
                        opt.map(|e| self.determine_symbols_expr(e, alias_manager, scope)),
                    ));
                }
                Stmt::If {
                    condition,
                    then_stmt,
                    else_stmt,
                } => {
                    let condition = self.determine_symbols_expr(condition, alias_manager, scope);
                    let then_stmt = Box::new(match *then_stmt {
                        Stmt::Block(stmts) => {
                            let mut inner = scope.clone();
                            Stmt::Block(self.determine_symbols_stmts_second_pass(
                                stmts,
                                &mut inner,
                                alias_manager,
                            ))
                        }
                        s => s,
                    });
                    let else_stmt = else_stmt.map(|s| {
                        Box::new(match *s {
                            Stmt::Block(stmts) => {
                                let mut inner = scope.clone();
                                Stmt::Block(self.determine_symbols_stmts_second_pass(
                                    stmts,
                                    &mut inner,
                                    alias_manager,
                                ))
                            }
                            s => s,
                        })
                    });
                    out.push(Stmt::If {
                        condition,
                        then_stmt,
                        else_stmt,
                    });
                }
                Stmt::While { condition, body } => {
                    let condition = self.determine_symbols_expr(condition, alias_manager, scope);
                    let body = Box::new(match *body {
                        Stmt::Block(stmts) => {
                            let mut inner = scope.clone();
                            Stmt::Block(self.determine_symbols_stmts_second_pass(
                                stmts,
                                &mut inner,
                                alias_manager,
                            ))
                        }
                        s => s,
                    });
                    out.push(Stmt::While { condition, body });
                }
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => {
                    let mut inner = scope.clone();
                    let init = init.map(|s| {
                        Box::new(match *s {
                            Stmt::Block(stmts) => {
                                Stmt::Block(self.determine_symbols_stmts_second_pass(
                                    stmts,
                                    &mut inner,
                                    alias_manager,
                                ))
                            }
                            s => s,
                        })
                    });
                    let condition =
                        condition.map(|e| self.determine_symbols_expr(e, alias_manager, &inner));
                    let update =
                        update.map(|e| self.determine_symbols_expr(e, alias_manager, &inner));
                    let body = Box::new(match *body {
                        Stmt::Block(stmts) => {
                            Stmt::Block(self.determine_symbols_stmts_second_pass(
                                stmts,
                                &mut inner,
                                alias_manager,
                            ))
                        }
                        s => s,
                    });
                    out.push(Stmt::For {
                        init,
                        condition,
                        update,
                        body,
                    });
                }
                Stmt::Block(stmts) => {
                    let mut inner = scope.clone();
                    out.push(Stmt::Block(self.determine_symbols_stmts_second_pass(
                        stmts,
                        &mut inner,
                        alias_manager,
                    )));
                }
                Stmt::FunDecl {
                    name,
                    params,
                    return_type,
                    body,
                    attributes,
                } => {
                    let mut fn_scope = scope.clone();
                    for (p, _) in &params {
                        fn_scope.insert(p.clone());
                    }
                    out.push(Stmt::FunDecl {
                        name,
                        params,
                        return_type,
                        body: self.determine_symbols_stmts_second_pass(
                            body,
                            &mut fn_scope,
                            alias_manager,
                        ),
                        attributes,
                    });
                }
                other => out.push(other),
            }
        }
        out
    }
}

pub type CanonicalFile = String;
pub type Alias = String;

#[derive(Clone, Default, Debug)]
pub struct AliasManager {
    pub module_registry: HashMap<CanonicalFile, QuorFile>,
    pub module_ids: HashMap<CanonicalFile, usize>,
    module_count: usize,
}

impl AliasManager {
    pub fn register_module(&mut self, file: CanonicalFile) {
        self.module_registry.insert(
            file.clone(),
            QuorFile {
                file: file.clone(),
                module_id: self.module_count,
                ..Default::default()
            },
        );
        self.module_ids.insert(file, self.module_count);
        self.module_count += 1;
    }

    /// Get the QuorFile for a canonical path (for symbol lookups).
    pub fn get_module(&self, canonical_file: &CanonicalFile) -> Option<&QuorFile> {
        self.module_registry.get(canonical_file)
    }

    /// Mutable access for running first pass and updating symbols/aliases.
    pub fn get_module_mut(&mut self, canonical_file: &CanonicalFile) -> Option<&mut QuorFile> {
        self.module_registry.get_mut(canonical_file)
    }

    /// Resolve "namespace::name" using the current file's aliases. Returns the target
    /// module's symbols and the qualified name (e.g. "sin.3") for type analysis.
    pub fn resolve_for_lookup<'a>(
        &'a self,
        name: &str,
        current_file: &'a QuorFile,
    ) -> Option<(&'a ModuleSymbols, String)> {
        if !name.contains("::") {
            return Some((
                &current_file.symbols,
                format!("{name}.{}", current_file.module_id),
            ));
        }
        let parts: Vec<&str> = name.split("::").collect();
        let (namespace, base_name) = match parts.as_slice() {
            [ns, base] => (*ns, *base),
            _ => return None,
        };
        let canonical = current_file.aliases.get(namespace)?;
        let target = self.module_registry.get(canonical)?;
        let module_id = *self.module_ids.get(canonical)?;
        Some((&target.symbols, format!("{base_name}.{module_id}")))
    }
}
