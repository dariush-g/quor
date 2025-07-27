use aera::{lexer::Lexer, parser::Parser};

#[test]
fn test_function_declaration() {
    let source = r#"
            fun add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;
    let mut lexer = Lexer::new(source.to_owned());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = Parser::new(tokens);
        if let Ok(_ast) = parser.parse() {}
    }
}

#[test]
fn test_binary_expression() {
    let source = "let result: i32 = 1 + 2 * 3;";
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
