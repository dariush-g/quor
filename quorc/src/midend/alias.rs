use std::collections::HashMap;

use crate::frontend::ast::*;

#[derive(Clone, Debug, Default)]
pub struct ModuleSymbols {
    pub structs: HashMap<String, bool>,
    pub struct_fields: HashMap<String, Vec<(String, Type)>>,
    pub functions: HashMap<String, (Vec<Type>, Type, Vec<String>)>,
    pub globals: HashMap<String, Type>,
}

#[derive(Clone, Debug, Default)]
pub struct QuorFile {
    pub file: CanonicalFile,
    pub module_id: usize,
    pub symbols: ModuleSymbols,
    pub aliases: HashMap<Alias, CanonicalFile>,
}

impl QuorFile {
    pub fn determine_symbols_stmts_first_pass(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut named_stmts: Vec<Stmt> = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::AtDecl(decl, name, val, _, _) => {
                    if decl.as_str() == "const" {
                        self.symbols.globals.insert(
                            format!("{}.{}", name.unwrap(), self.module_id),
                            val.unwrap().get_type(),
                        );
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
                    self.symbols.functions.insert(
                        name.clone(),
                        (
                            params.iter().map(|(_, ty)| ty.clone()).collect(),
                            return_type.clone(),
                            attributes.clone(),
                        ),
                    );
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
                    self.symbols.structs.insert(name.clone(), union);
                    self.symbols
                        .struct_fields
                        .insert(name.clone(), instances.clone());

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
                } => todo!(),
                Stmt::While { condition, body } => todo!(),
                Stmt::For {
                    init,
                    condition,
                    update,
                    body,
                } => todo!(),
                Stmt::Block(stmts) => todo!(),
                _ => named_stmts.push(stmt),
            }
        }

        named_stmts
    }

    pub fn determine_symbols_expr(expr: Expr) -> Expr {
        match expr {
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
            _ => expr.clone(),
        }
    }
}

pub type CanonicalFile = String;
pub type Alias = String;

#[derive(Default, Debug)]
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
}
