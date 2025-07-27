#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    int,
    float,
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

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    CharLiteral(char),

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
