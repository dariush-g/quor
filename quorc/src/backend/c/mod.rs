use std::collections::HashMap;

use crate::frontend::ast::{Stmt, Type};

#[derive(Clone, Debug)]
struct CStruct {
    name: String,
    fields: Vec<(String, String)>, // name type
}

#[derive(Clone, Debug)]
struct CUnion {
    name: String,
    fields: Vec<(String, String)>, // name type
}

#[derive(Clone, Debug)]
struct CFunc {
    name: String,
    params: Vec<(String, String)>, // name type
    body: String,
    ret_type: String,
}

#[derive(Clone, Debug)]
struct CGlobal {
    name: String,
    ty: String,
    val: String,
}

#[derive(Clone, Debug)]
pub struct CEmitter {
    structs: Vec<CStruct>,
    unions: Vec<CUnion>,
    functions: Vec<CFunc>,
    globals: Vec<CGlobal>,
}

impl CEmitter {
    fn define_func(&mut self, stmt: Stmt) {}
    fn define_struct(&mut self, stmt: Stmt) {}
    fn define_global(&mut self, stmt: Stmt) {}
    fn complete_c(&self) {}
    fn compile_c(&self) {}
}
