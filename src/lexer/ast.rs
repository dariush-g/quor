use crate::lexer::token::{FloatType, IntType};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    u128,
    f32,
    f64,
    Bool,
    Array(Box<Type>, usize),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Void,
    Unknown,
    Char,
}

impl Type {
    pub fn from_int_type(int_type: crate::lexer::token::IntType) -> Self {
        use crate::lexer::token::IntType;
        match int_type {
            IntType::i8 => Type::i8,
            IntType::i16 => Type::i16,
            IntType::i32 => Type::i32,
            IntType::i64 => Type::i64,
            IntType::i128 => Type::i128,
            IntType::u8 => Type::u8,
            IntType::u16 => Type::u16,
            IntType::u32 => Type::u32,
            IntType::u64 => Type::u64,
            IntType::u128 => Type::u128,
        }
    }

    pub fn from_float_type(float_type: crate::lexer::token::FloatType) -> Self {
        use crate::lexer::token::FloatType;
        match float_type {
            FloatType::f32 => Type::f32,
            FloatType::f64 => Type::f64,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64, IntType),
    FloatLiteral(f64, FloatType),
    BoolLiteral(bool),
    
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        result_type: Type,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        result_type: Type,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        return_type: Type,
    },
    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },
    Array(Vec<Expr>, Type),
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
        element_type: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,    // !
    Negate, // -
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        name: String,
        var_type: Type,
        value: Expr,
    },
    FunDecl {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },

    Block(Vec<Stmt>),
    Expression(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
}
