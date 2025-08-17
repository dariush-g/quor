use crate::lexer::token::{Token, TokenType};

pub mod ast;
pub mod token;

pub struct Lexer {
    pub source: String,
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let chars = source.chars().collect();
        Self {
            source,
            chars,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(token_type, self.line, self.column, self.current)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if !self.is_at_end() {
                self.scan_token(&mut tokens)?;
                // println!("Token {:?}", self.peek())
            }
        }

        tokens.push(self.make_token(TokenType::Eof));
        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '/' if self.peek_next() == '/' => {
                    // Skip until end of line
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_token(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexError> {
        let c = self.advance();

        match c {
            '@' => tokens.push(self.make_token(TokenType::At)),
            '\'' => tokens.push(self.scan_char()?),
            '"' => {
                let mut string = String::new();
                let mut ch = self.advance();
                while ch != '"' {
                    string.push(ch);
                    ch = self.advance();
                }
                tokens.push(self.make_token(TokenType::StringLiteral(string)));
            }
            '.' => tokens.push(self.make_token(TokenType::Period)),

            '(' => tokens.push(self.make_token(TokenType::LeftParen)),
            ')' => tokens.push(self.make_token(TokenType::RightParen)),
            '{' => tokens.push(self.make_token(TokenType::LeftBrace)),
            '}' => tokens.push(self.make_token(TokenType::RightBrace)),
            '[' => tokens.push(self.make_token(TokenType::LeftBracket)),
            ']' => tokens.push(self.make_token(TokenType::RightBracket)),
            ',' => tokens.push(self.make_token(TokenType::Comma)),
            ';' => tokens.push(self.make_token(TokenType::Semicolon)),
            '+' => tokens.push(self.make_token(TokenType::Plus)),
            '*' => tokens.push(self.make_token(TokenType::Star)),
            '%' => tokens.push(self.make_token(TokenType::Percent)),
            '!' => {
                let token = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                tokens.push(self.make_token(token));
            }
            ':' => {
                let token = if self.match_char(':') {
                    TokenType::DoubleColon
                } else {
                    TokenType::Colon
                };

                tokens.push(self.make_token(token))
            }

            '=' => {
                let token = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                tokens.push(self.make_token(token));
            }
            '<' => {
                let token = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                tokens.push(self.make_token(token));
            }
            '>' => {
                let token = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                tokens.push(self.make_token(token));
            }
            '-' => {
                let token = if self.match_char('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                };
                tokens.push(self.make_token(token));
            }
            '&' => {
                if self.match_char('&') {
                    tokens.push(self.make_token(TokenType::And));
                } else {
                    tokens.push(self.make_token(TokenType::Ampersand));
                }
            }
            '|' => {
                if self.match_char('|') {
                    tokens.push(self.make_token(TokenType::Or));
                } else {
                    return Err(LexError::InvalidCharacter(c, self.line, self.column));
                }
            }
            '/' => {
                // We've already handled comments in skip_whitespace
                tokens.push(self.make_token(TokenType::Slash));
            }
            c if c.is_ascii_digit() => {
                tokens.push(self.scan_number()?);
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                tokens.push(self.scan_identifier());
            }
            _ => {
                return Err(LexError::InvalidCharacter(c, self.line, self.column));
            }
        }

        Ok(())
    }

    fn scan_identifier(&mut self) -> Token {
        let start_line = self.line;
        let start_col = self.column - 1; // -1 because we already advanced
        let start_pos = self.current - 1;

        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text: String = self.chars[start_pos..self.current].iter().collect();

        let token_type = match text.as_str() {
            "def" => TokenType::Def,

            "let" => TokenType::Let,
            "if" => TokenType::If,

            "class" => TokenType::Class,

            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "as" => TokenType::As,

            "int" => TokenType::Int,
            "float" => TokenType::Float,
            "bool" => TokenType::Boolean,
            "void" => TokenType::Void,
            "char" => TokenType::Char,
            _ => TokenType::Identifier(text),
        };

        Token::new(token_type, start_line, start_col, start_pos)
    }

    fn scan_number(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column - 1; // already advanced
        let start_pos = self.current - 1;

        let mut is_float = false;

        // Consume integer part
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance(); // consume '.'

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num_text: String = self.chars[start_pos..self.current].iter().collect();

        if is_float {
            let value = num_text
                .parse::<f32>()
                .map_err(|_| LexError::InvalidNumber(num_text.clone(), start_line, start_col))?;

            Ok(Token::new(
                TokenType::FloatLiteral(value),
                start_line,
                start_col,
                start_pos,
            ))
        } else {
            let value = num_text
                .parse::<i32>()
                .map_err(|_| LexError::InvalidNumber(num_text.clone(), start_line, start_col))?;

            Ok(Token::new(
                TokenType::IntLiteral(value),
                start_line,
                start_col,
                start_pos,
            ))
        }
    }

    fn scan_char(&mut self) -> Result<Token, LexError> {
        let start_line = self.line;
        let start_col = self.column - 1; // we already consumed the opening quote
        let start_pos = self.current - 1;

        if self.is_at_end() || self.peek() == '\n' {
            return Err(LexError::InvalidCharacter('\'', start_line, start_col));
        }

        // Read the character (support common escapes)
        let ch = match self.advance() {
            '\\' => {
                // escaped sequence
                let esc = if self.is_at_end() {
                    '\0'
                } else {
                    self.advance()
                };
                match esc {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '0' => '\0',
                    'x' => {
                        // Optional: \xHH (two hex digits), fallback to error if malformed
                        let h1 = self.peek();
                        let h2 = self.peek_next();
                        if h1.is_ascii_hexdigit() && h2.is_ascii_hexdigit() {
                            // consume both
                            let _ = self.advance();
                            let _ = self.advance();
                            let hex = format!("{h1}{h2}");
                            let val = u8::from_str_radix(&hex, 16).map_err(|_| {
                                LexError::InvalidCharacter('x', start_line, start_col)
                            })?;
                            val as char
                        } else {
                            return Err(LexError::InvalidCharacter('x', start_line, start_col));
                        }
                    }
                    _ => return Err(LexError::InvalidCharacter(esc, start_line, start_col)),
                }
            }
            c => c,
        };

        // Expect closing quote
        if self.peek() != '\'' {
            return Err(LexError::InvalidCharacter('\'', start_line, start_col));
        }
        self.advance(); // consume closing quote

        Ok(Token::new(
            TokenType::CharLiteral(ch),
            start_line,
            start_col,
            start_pos,
        ))
    }

    fn advance(&mut self) -> char {
        if !self.is_at_end() {
            let c = self.chars[self.current];
            self.current += 1;
            self.column += 1;
            c
        } else {
            '\0'
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}

#[derive(Debug)]
pub enum LexError {
    InvalidCharacter(char, usize, usize),
    InvalidNumber(String, usize, usize),
    InvalidTypeSuffix(String, usize, usize),
}

// use crate::lexer::token::{FloatType, IntType, Token, TokenType};

// pub mod ast;
// pub mod token;

// pub struct Lexer {
//     _source: String,
//     chars: Vec<char>,
//     current: usize,
//     line: usize,
//     column: usize,
// }

// impl Lexer {
//     pub fn new(_source: String) -> Self {
//         let chars = _source.chars().collect();
//         Self {
//             _source,
//             chars,
//             column: 1,
//             line: 1,
//             current: 0,
//         }
//     }

//     fn make_token(&self, token_type: TokenType) -> Token {
//         Token::new(token_type, self.line, self.column, self.current)
//     }

//     pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
//         let mut tokens = Vec::new();

//         while !self.is_at_end() {
//             self.scan_token(&mut tokens)?;
//         }

//         tokens.push(Token::new(
//             TokenType::Eof,
//             self.line,
//             self.column,
//             self.current,
//         ));
//         Ok(tokens)
//     }

//     fn scan_token(&mut self, tokens: &mut Vec<Token>) -> Result<(), LexError> {
//         let c = self.advance();

//         match c {
//             ' ' | '\r' | '\t' => {}
//             '\n' => {
//                 tokens.push(self.make_token(TokenType::Newline));
//                 self.line += 1;
//                 self.column = 1;
//             }
//             '(' => tokens.push(self.make_token(TokenType::LeftParen)),
//             ')' => tokens.push(self.make_token(TokenType::RightParen)),
//             '{' => tokens.push(self.make_token(TokenType::LeftBrace)),
//             '}' => tokens.push(self.make_token(TokenType::RightBrace)),
//             '[' => tokens.push(self.make_token(TokenType::LeftBracket)),
//             ']' => tokens.push(self.make_token(TokenType::RightBracket)),
//             ',' => tokens.push(self.make_token(TokenType::Comma)),
//             ';' => tokens.push(self.make_token(TokenType::Semicolon)),
//             ':' => tokens.push(self.make_token(TokenType::Colon)),
//             '+' => tokens.push(self.make_token(TokenType::Plus)),
//             '*' => tokens.push(self.make_token(TokenType::Star)),
//             '%' => tokens.push(self.make_token(TokenType::Percent)),
//             '!' => {
//                 if self.peek() == '=' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::BangEqual));
//                 } else {
//                     tokens.push(self.make_token(TokenType::Bang));
//                 }
//             }
//             '=' => {
//                 if self.peek() == '=' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::EqualEqual));
//                 } else {
//                     tokens.push(self.make_token(TokenType::Equal));
//                 }
//             }
//             '<' => {
//                 if self.peek() == '=' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::LessEqual));
//                 } else {
//                     tokens.push(self.make_token(TokenType::Less));
//                 }
//             }
//             '>' => {
//                 if self.peek() == '=' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::GreaterEqual));
//                 } else {
//                     tokens.push(self.make_token(TokenType::Greater));
//                 }
//             }
//             '-' => {
//                 if self.peek() == '>' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::Arrow));
//                 } else {
//                     tokens.push(self.make_token(TokenType::Minus));
//                 }
//             }
//             '&' => {
//                 if self.peek() == '&' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::And));
//                 } else {
//                     return Err(LexError::InvalidCharacter(c, self.line, self.column));
//                 }
//             }
//             '|' => {
//                 if self.peek() == '|' {
//                     self.advance();
//                     tokens.push(self.make_token(TokenType::Or));
//                 } else {
//                     return Err(LexError::InvalidCharacter(c, self.line, self.column));
//                 }
//             }

//             // Slash or comment
//             '/' => {
//                 if self.peek() == '/' {
//                     // Line comment - skip until end of line
//                     while self.peek() != '\n' && !self.is_at_end() {
//                         self.advance();
//                     }
//                 } else {
//                     tokens.push(self.make_token(TokenType::Slash));
//                 }
//             }
//             // Numbers
//             c if c.is_ascii_digit() => {
//                 tokens.push(self.scan_number()?);
//             }

//             // Identifiers and keywords
//             c if c.is_ascii_alphabetic() || c == '_' => {
//                 tokens.push(self.scan_identifier());
//             }

//             _ => {
//                 return Err(LexError::InvalidCharacter(c, self.line, self.column));
//             }
//         }

//         Ok(())
//     }

//     fn scan_identifier(&mut self) -> Token {
//         let start_line = self.line;
//         let start_col = self.column;
//         let start_pos = self.current - 1;

//         self.current -= 1;
//         self.column -= 1;

//         while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
//             self.advance();
//         }

//         let text: String = self.chars[start_pos..self.current].iter().collect();

//         let token_type = match text.as_str() {
//             "let" => TokenType::Let,
//             "fn" => TokenType::Fn,
//             "else" => TokenType::Else,
//             "while" => TokenType::While,
//             "for" => TokenType::For,
//             "return" => TokenType::Return,
//             "true" => TokenType::True,
//             "false" => TokenType::False,
//             "as" => TokenType::As,

//             "i8" => TokenType::i8,
//             "i16" => TokenType::i16,
//             "i32" => TokenType::i32,
//             "i64" => TokenType::i64,
//             "i128" => TokenType::i128,
//             "u8" => TokenType::u8,
//             "u16" => TokenType::u16,
//             "u32" => TokenType::u32,
//             "u64" => TokenType::u64,
//             "u128" => TokenType::u128,
//             "f32" => TokenType::f32,
//             "f64" => TokenType::f64,
//             "bool" => TokenType::Boolean,

//             _ => TokenType::Identifier(text),
//         };

//         Token::new(token_type, start_line, start_col, start_pos)
//     }

//     fn scan_number(&mut self) -> Result<Token, LexError> {
//         let start_line = self.line;
//         let start_column = self.column;
//         let start_pos = self.current - 1;

//         self.current -= 1;
//         self.column -= 1;

//         while self.peek().is_ascii_digit() {
//             self.advance();
//         }

//         let mut is_float = false;

//         if self.peek() == '.' && self.peek_next().is_ascii_digit() {
//             is_float = true;
//             self.advance();

//             while self.peek().is_ascii_digit() {
//                 self.advance();
//             }
//         }

//         let number_end = self.current;
//         let mut suffix = String::new();

//         if self.peek().is_ascii_alphanumeric() {
//             while self.peek().is_ascii_alphanumeric() {
//                 suffix.push(self.advance());
//             }
//         }

//         let number_text: String = self.chars[start_pos..number_end].iter().collect();

//         if is_float {
//             let value: f64 = number_text
//                 .parse()
//                 .map_err(|_| LexError::InvalidNumber(start_line, start_column))?;

//             let float_type = match suffix.as_str() {
//                 "" => None,
//                 "f32" => Some(FloatType::f32),
//                 "f64" => Some(FloatType::f64),
//                 _ => return Err(LexError::InvalidTypeSuffix(start_line, start_column)),
//             };

//             Ok(Token::new(
//                 TokenType::FloatLiteral(value, float_type),
//                 start_line,
//                 start_column,
//                 start_pos,
//             ))
//         } else {
//             let value: i64 = number_text
//                 .parse()
//                 .map_err(|_| LexError::InvalidNumber(start_line, start_column))?;

//             let int_type = match suffix.as_str() {
//                 "" => None,
//                 "i8" => Some(IntType::i8),
//                 "i16" => Some(IntType::i16),
//                 "i32" => Some(IntType::i32),
//                 "i64" => Some(IntType::i64),
//                 "i128" => Some(IntType::i128),
//                 "u8" => Some(IntType::u8),
//                 "u16" => Some(IntType::u16),
//                 "u32" => Some(IntType::u32),
//                 "u64" => Some(IntType::u64),
//                 "u128" => Some(IntType::u128),
//                 _ => return Err(LexError::InvalidTypeSuffix(start_line, start_column)),
//             };

//             Ok(Token::new(
//                 TokenType::IntLiteral(value, int_type),
//                 start_line,
//                 start_column,
//                 start_pos,
//             ))
//         }
//     }

//     fn advance(&mut self) -> char {
//         if !self.is_at_end() {
//             let c = self.chars[self.current];
//             self.current += 1;
//             self.column += 1;
//             c
//         } else {
//             '\0'
//         }
//     }

//     fn peek(&self) -> char {
//         if self.is_at_end() {
//             '\0'
//         } else {
//             self.chars[self.current]
//         }
//     }

//     fn peek_next(&self) -> char {
//         if self.current + 1 >= self.chars.len() {
//             '\0'
//         } else {
//             self.chars[self.current + 1]
//         }
//     }

//     fn is_at_end(&self) -> bool {
//         self.current >= self.chars.len()
//     }
// }

// pub enum LexError {
//     InvalidCharacter(char, usize, usize),
//     InvalidNumber(usize, usize),
//     InvalidTypeSuffix(usize, usize),
// }
