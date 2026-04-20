use crate::frontend::ast::Type;

#[derive(Clone, Debug)]
pub enum SizeOf {
    Variable(String),
    Prim(Type),
}
