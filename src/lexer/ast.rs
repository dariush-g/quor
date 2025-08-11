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

    Pointer(Box<Type>),
}

impl Type {
    pub fn pointer_to(base: Type) -> Type {
        Type::Pointer(Box::new(base))
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    pub fn deref(&self) -> Option<&Type> {
        match self {
            Type::Pointer(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    CharLiteral(char),

    AddressOf(Box<Expr>), // &expr

    DerefAssign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

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
        //element_type: Type,
    },
}

impl Expr {
    pub fn get_type(&self) -> Type {
        match self {
            Expr::IntLiteral(_) => Type::int,
            Expr::FloatLiteral(_) => Type::float,
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::CharLiteral(_) => Type::Char,
            Expr::Variable(_) => Type::Unknown,
            Expr::Binary { result_type, .. } => result_type.clone(),
            Expr::Unary { result_type, .. } => result_type.clone(),
            Expr::Call { return_type, .. } => return_type.clone(),
            Expr::Cast { target_type, .. } => target_type.clone(),
            Expr::AddressOf(expr) => Type::Pointer(Box::new(expr.get_type())),
            Expr::DerefAssign { target, .. } => target.get_type(),
            Expr::Array(_, element_type) => Type::Array(Box::new(element_type.clone()), 0),
            _ => Type::Unknown,
        }
    }
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
    Not,         // !
    Negate,      // -
    AddressOf,   // &
    Dereference, // *
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
