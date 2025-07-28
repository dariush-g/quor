use std::fs;
use std::path::Path;

use aera::analyzer::TypeChecker;
use aera::codegen::CodeGen;
use aera::lexer::Lexer;
use aera::parser::Parser;

fn main() {
    // 1. Read source file
    let path = Path::new("test.ae");
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            return;
        }
    };

    // 2. Lexing
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {:?}", e);
            return;
        }
    };

    // 3. Parsing
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Parser error: {:?}", e);
            return;
        }
    };

    // 4. Type checking
    let typed = match TypeChecker::analyze_program(program) {
        Ok(typed_program) => typed_program,
        Err(e) => {
            eprintln!("Type error: {:?}", e);
            return;
        }
    };

    let code = CodeGen::generate(&typed);

    let _ = std::fs::write("test.asm", code);
}
