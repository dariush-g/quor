use std::{fs, path::Path};

use crate::{lexer::Lexer, parser::Parser};

pub mod lexer;
pub mod parser;

fn main() {
    let path = Path::new("test.ae");

    let source = fs::read_to_string(path).expect("Failed to read file");

    let mut lexer = Lexer::new(source.to_owned());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(_ast) = parser.parse() {}
    }
    let mut lexer = Lexer::new(source.to_owned());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(ast) = parser.parse() {
            for stmt in ast {
                println!("{stmt:?}");
            }
        }
    }
}
