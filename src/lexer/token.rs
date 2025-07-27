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
    IntLiteral(i64, Option<IntType>),
    FloatLiteral(f64, Option<FloatType>),
    Boolean,

    Var,
    Fn,
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

    // Special
    Newline,
    Eof,

    Identifier(String),

    Void,
    Char,
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
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum IntType {
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
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum FloatType {
    f32,
    f64,
}
