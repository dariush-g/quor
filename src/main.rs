use std::fs;
use std::path::Path;

use aera::analyzer::TypeChecker;
use aera::lexer::Lexer;
use aera::parser::Parser;

fn main() {
    let path = Path::new("test.ae");
    let source = fs::read_to_string(path).expect("Failed to read file");

    // Lexing
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Lexer error: {:?}", e);
            return;
        }
    };

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parser error: {:?}", e);
            return;
        }
    };

    // Type checking
    match TypeChecker::analyze_program(program) {
        Ok(typed_program) => {
            println!("Program is well-typed!");
            
        }
        Err(e) => {
            eprintln!("Type error: {}", e);
        }
    }
}
