use crate::lexer::ast::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, position: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            position,
        }
    }

    pub fn simple(token_type: TokenType, position: usize) -> Self {
        Self {
            token_type,
            line: 0,
            column: 0,
            position,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    IntLiteral(i32),
    FloatLiteral(f32),
    CharLiteral(char),
    StringLiteral(String),
    LongLiteral(i64),

    Struct,

    Boolean,

    Let,
    Def,
    If,
    Else,
    While,
    Return,
    True,
    False,
    For,

    And,
    Or,
    As,

    // Operators
    At,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Arrow,
    DoubleColon,
    Ampersand,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Period,
    Colon,

    SingleQuote,
    DoubleQuote,

    Newline,

    // end of file
    Eof,

    // names, etc
    Identifier(String),

    Void,
    Char,
    Float,
    Int,
    Long,
}

impl TokenType {
    pub fn get_type(&self) -> Type {
        match self {
            TokenType::IntLiteral(_) => Type::int,
            TokenType::FloatLiteral(_) => Type::float,
            TokenType::CharLiteral(_) => Type::Char,
            TokenType::Struct => Type::Struct {
                name: String::new(),
                instances: Vec::new(),
            },
            TokenType::Boolean => Type::Bool,

            TokenType::Void => Type::Void,
            TokenType::Char => Type::Char,
            TokenType::Float => Type::float,
            TokenType::Int => Type::int,
            _ => Type::Unknown,
        }
    }
}
