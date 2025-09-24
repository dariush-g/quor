#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    int,
    float,
    Long,
    Char,

    null,

    Bool,

    Array(Box<Type>, Option<usize>),

    Function,

    Struct {
        name: String,
        instances: Vec<(String, Type)>,
    },

    Void,

    Unknown,

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

    pub fn size(&self) -> usize {
        match self {
            Type::int => 4,
            Type::float => 4,
            Type::Char => 1,
            Type::Bool => 1,
            Type::Pointer(_) => 8,
            Type::Array(elem, len) => elem.size() * len.unwrap_or(0),
            Type::Struct { instances, .. } => instances.iter().map(|f| f.1.size()).sum(),
            Type::Long => 8,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i32),
    LongLiteral(i64),
    FloatLiteral(f32),
    BoolLiteral(bool),
    StringLiteral(String),
    // name of class
    CharLiteral(char),

    // UnionInit {
    //     name: String,
    //     param: (String, Box<Expr>),
    // },
    StructInit {
        name: String,
        params: Vec<(String, Expr)>,
    },

    AddressOf(Box<Expr>), // &expr

    DerefAssign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    //          class, iname
    InstanceVar(String, String),

    Variable(String, Type),

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

    IndexAssign {
        array: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },

    FieldAssign {
        class_name: String,
        field: String,
        value: Box<Expr>,
    },
}

impl Expr {
    pub fn get_type(&self) -> Type {
        match self {
            Expr::StructInit { name, params } => Type::Struct {
                name: name.to_string(),
                instances: params
                    .iter()
                    .map(|(name, expr)| (name.clone(), expr.get_type()))
                    .collect(),
            },
            Expr::IntLiteral(_) => Type::int,
            Expr::FloatLiteral(_) => Type::float,
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::CharLiteral(_) => Type::Char,
            Expr::Variable(_, ty) => ty.clone(),
            Expr::Binary { result_type, .. } => result_type.clone(),
            Expr::Unary { expr, .. } => expr.get_type(),
            Expr::Call { return_type, .. } => return_type.clone(),
            Expr::Cast { target_type, .. } => target_type.clone(),
            Expr::AddressOf(expr) => Type::Pointer(Box::new(expr.get_type())),
            Expr::DerefAssign { target, .. } => target.get_type(),
            Expr::Array(elements, element_type) => {
                Type::Array(Box::new(element_type.clone()), Some(elements.len()))
            }
            Expr::StringLiteral(_) => Type::Pointer(Box::new(Type::Char)),
            Expr::IndexAssign { value, .. } => value.get_type(),
            // Expr::InstanceVar(_, _) => todo!(),
            Expr::Assign { value, .. } => value.get_type(),
            // Expr::ArrayAccess { array, index } => todo!(),
            // Expr::FieldAssign { class_name, field, value } => todo!(),
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
    AtDecl(String, Option<String>, Option<Expr>),
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
    StructDecl {
        name: String,
        instances: Vec<(String, Type)>,
        union: bool,
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
