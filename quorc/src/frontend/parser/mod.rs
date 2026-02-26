use std::collections::HashSet;

use crate::frontend::{ast::*, lexer::token::*};

pub struct Parser {
    tokens: Vec<Token>,
    current_generics: HashSet<String>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            current_generics: HashSet::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            while self.match_token(&[TokenType::Newline]) {}

            if !self.is_at_end() {
                statements.push(self.statement(true, None)?);
            }
        }

        Ok(statements)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError::Expected {
                expected: token_type,
                found: self.peek().clone(),
                message: message.to_string(),
            })
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        let mut epilogue = "";

        let path = match &self.peek().token_type {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            TokenType::Less => {
                self.advance(); // <
                let mut path = String::new();
                epilogue = "!";
                loop {
                    match &self.peek().token_type {
                        TokenType::Identifier(s) => {
                            path.push_str(s);
                            self.advance();
                        }
                        TokenType::Period => {
                            path.push('.');
                            self.advance();
                        }
                        TokenType::Greater => {
                            self.advance();
                            break;
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(self.peek().clone()));
                        }
                    }
                }

                path
            }

            _ => return Err(ParseError::UnexpectedToken(self.peek().clone())),
        };

        let path = format!("{path}{epilogue}");
        Ok(Stmt::AtDecl("import".to_string(), Some(path), None, None))
    }

    fn at_declaration(&mut self) -> Result<Stmt, ParseError> {
        if let TokenType::Identifier(decl) = &self.peek().token_type.clone() {
            self.advance();

            // if let TokenType::LeftParen = self.peek().token_type {
            //     let str = self.expression()?;

            //     self.current -= 1;

            //     match str {
            //         Expr::StringLiteral(param) => {
            //             let stmt = Stmt::AtDecl(decl.to_string(), Some(param));

            //             self.consume(TokenType::RightParen, "Expected ')'")?;

            //             self.consume(TokenType::Semicolon, "Expected ';'")?;
            //             return Ok(stmt);
            //         }
            //         _ => {
            //             return Err(ParseError::Expected {
            //                 expected: TokenType::Identifier("declaration".to_string()),
            //                 found: self.peek().clone(),
            //                 message: "Expected declaration after '@'".to_owned(),
            //             });
            //         }
            //     }
            // }

            // if let TokenType::DoubleColon = self.peek().token_type {
            //     self.advance();
            // if let TokenType::Less = self.peek().token_type {
            //     self.advance(); // consume '('
            //     let mut param = if let TokenType::Identifier(s) = &self.peek().token_type {
            //         let val = s.clone();
            //         self.advance(); // consume string
            //         val
            //     } else {
            //         return Err(ParseError::Expected {
            //             expected: TokenType::Identifier("".to_string()),
            //             found: self.peek().clone(),
            //             message: "Expected identifier in @import".to_owned(),
            //         });
            //     };

            //     if let TokenType::Period = self.peek().token_type {
            //         param.push('.');
            //         self.advance();
            //         if let TokenType::Identifier(qu) = &self.peek().token_type {
            //             param.push_str(qu);
            //             self.advance();
            //         }
            //     }

            //     param.push('!');

            //     self.consume(TokenType::Greater, "Expected '>'")?;
            //     return Ok(Stmt::AtDecl(decl.to_string(), Some(param), None, None));
            // }
            // // if let TokenType::LeftParen = self.peek().token_type {
            // // self.advance(); // consume '('
            // if let TokenType::StringLiteral(s) = &self.peek().clone().token_type {
            //     self.advance();

            //     return Ok(Stmt::AtDecl(
            //         decl.to_string(),
            //         Some(s.to_string()),
            //         None,
            //         None,
            //     ));
            // }

            if decl == "import" {
                return self.parse_import();
            }

            if decl == "extern"
                && let TokenType::Identifier(name) = &self.peek().token_type.clone()
            {
                self.advance();
                return Ok(Stmt::AtDecl(
                    decl.to_string(),
                    Some(name.to_string()),
                    None,
                    None,
                ));
            }

            if decl == "cfg" {
                if let TokenType::LeftBracket = self.peek().token_type {
                    self.advance();
                    let mut cfg = vec![];
                    while TokenType::RightBracket != self.peek().token_type {
                        cfg.push(self.peek().clone());
                        self.advance();
                    }
                    self.advance();

                    let stmt = self.parse_cfg(cfg);

                    return Ok(Stmt::AtDecl(
                        decl.to_string(),
                        None,
                        None,
                        Some(Box::new(stmt?)),
                    ));
                }
                return Err(ParseError::UnexpectedToken(self.peek().clone()));
            }

            if let TokenType::Identifier(name) = &self.peek().clone().token_type {
                self.advance();

                if let TokenType::Equal = &self.peek().token_type {
                    self.advance();
                    let expr = self.expression().unwrap_or_else(|_| panic!());
                    return Ok(Stmt::AtDecl(
                        decl.to_string(),
                        Some(name.to_string()),
                        Some(expr),
                        None,
                    ));
                }
            }

            // if let TokenType::LeftBrace = &self.peek().token_type {
            //     self.advance();
            //     let mut lines = Vec::new();
            //     while self.peek().token_type != TokenType::RightBrace {
            //         if let TokenType::Identifier(str) = &self.peek().token_type {
            //             lines.push(str.clone());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::IntLiteral(str) = &self.peek().token_type {
            //             lines.push(str.to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::CharLiteral(str) = &self.peek().token_type {
            //             lines.push(str.to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::FloatLiteral(str) = &self.peek().token_type {
            //             lines.push(str.to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::StringLiteral(str) = &self.peek().token_type {
            //             lines.push(str.to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::LongLiteral(str) = &self.peek().token_type {
            //             lines.push(str.to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         if let TokenType::Comma = &self.peek().token_type {
            //             lines.push(",".to_string());
            //             lines.push("\n".to_owned());
            //         }
            //         self.advance();
            //     }
            //     self.advance();
            //     #[allow(suspicious_double_ref_op)]
            //     return Ok(Stmt::AtDecl(
            //         decl.to_string(),
            //         Some(lines.iter().map(|str| str.clone().clone()).collect()),
            //         None,
            //         None,
            //     ));
            // }

            if let TokenType::LeftBrace = &self.peek().token_type {
                self.advance();
                let mut assembly_code = String::new();

                while self.peek().token_type != TokenType::RightBrace {
                    // println!("Processing token: {:?}", self.peek().token_type);
                    let current_token = self.peek().token_type.clone();

                    match &current_token {
                        TokenType::Identifier(str) => {
                            assembly_code.push_str(str);
                        }
                        TokenType::IntLiteral(num) => {
                            assembly_code.push_str(&num.to_string());
                        }
                        TokenType::CharLiteral(ch) => {
                            assembly_code.push('\'');
                            assembly_code.push(*ch);
                            assembly_code.push('\'');
                        }
                        TokenType::FloatLiteral(float) => {
                            assembly_code.push_str(&float.to_string());
                        }
                        TokenType::StringLiteral(string) => {
                            assembly_code.push_str(string);
                        }
                        TokenType::LongLiteral(long) => {
                            assembly_code.push_str(&long.to_string());
                        }
                        TokenType::Comma => {
                            assembly_code.push(',');
                        }
                        TokenType::Newline => {
                            assembly_code.push('\n');
                        }
                        TokenType::Plus => {
                            assembly_code.push('+');
                        }
                        TokenType::Minus => {
                            assembly_code.push('-');
                        }
                        TokenType::Star => {
                            assembly_code.push('*');
                        }
                        TokenType::Hashtag => {
                            assembly_code.push('#');
                        }
                        TokenType::Slash => {
                            assembly_code.push('/');
                        }
                        TokenType::Equal => {
                            assembly_code.push('=');
                        }
                        TokenType::LeftParen => {
                            assembly_code.push('(');
                        }
                        TokenType::RightParen => {
                            assembly_code.push(')');
                        }
                        TokenType::LeftBracket => {
                            assembly_code.push('[');
                        }
                        TokenType::RightBracket => {
                            assembly_code.push(']');
                        }
                        TokenType::Semicolon => {
                            assembly_code.push(';');
                        }
                        TokenType::Colon => {
                            assembly_code.push(':');
                        }
                        TokenType::Period => {
                            assembly_code.push('.');
                        }
                        TokenType::Ampersand => {
                            assembly_code.push('&');
                        }
                        TokenType::Bang => {
                            assembly_code.push('!');
                        }
                        TokenType::Greater => {
                            assembly_code.push('>');
                        }
                        TokenType::Less => {
                            assembly_code.push('<');
                        }
                        TokenType::Percent => {
                            assembly_code.push('%');
                        }
                        _ => {
                            // Add a space for unknown tokens to maintain readability
                            assembly_code.push(' ');
                        }
                    }

                    self.advance();

                    // Add a space after most tokens, but be smart about it
                    if !self.is_at_end() && self.peek().token_type != TokenType::RightBrace {
                        let next_token = &self.peek().token_type;

                        // Don't add space after comma, newline, or closing punctuation
                        if !matches!(
                            current_token,
                            TokenType::Comma
                                | TokenType::Newline
                                | TokenType::RightParen
                                | TokenType::RightBracket
                                | TokenType::Semicolon
                        ) {
                            // Don't add space before newline or closing punctuation
                            if !matches!(
                                next_token,
                                TokenType::Newline
                                    | TokenType::RightParen
                                    | TokenType::RightBracket
                                    | TokenType::Semicolon
                                    | TokenType::RightBrace
                            ) {
                                assembly_code.push(' ');
                            }
                        }
                    }
                }
                self.advance();

                return Ok(Stmt::AtDecl(
                    decl.to_string(),
                    Some(assembly_code),
                    None,
                    None,
                ));
            }

            return Ok(Stmt::AtDecl(decl.to_string(), None, None, None));
        }

        Err(ParseError::Expected {
            expected: TokenType::Identifier("declaration".to_string()),
            found: self.peek().clone(),
            message: "Expected declaration after '@'".to_owned(),
        })
    }

    fn parse_cfg(&mut self, tokens: Vec<Token>) -> Result<Stmt, ParseError> {
        let mut pos = 0;
        let expr = self.parse_cfg_expr(&tokens, &mut pos);
        if let TokenType::LeftBrace = self.peek().token_type {
            self.advance();
            let block = Stmt::Block(self.block(None)?);
            return Ok(Stmt::CfgStmt(expr, Box::new(block)));
        }
        Err(ParseError::Expected {
            expected: TokenType::LeftBrace,
            found: self.peek().clone(),
            message: "Expected block after cfg declaration".to_owned(),
        })
    }

    fn parse_cfg_expr(&self, tokens: &[Token], pos: &mut usize) -> CfgExpr {
        let mut left = self.parse_cfg_cmp(tokens, pos);

        while *pos < tokens.len() {
            match &tokens[*pos].token_type {
                TokenType::Ampersand => {
                    *pos += 1;
                    let right = self.parse_cfg_cmp(tokens, pos);
                    left = CfgExpr::Cmp {
                        left: Box::new(left),
                        right: Box::new(right),
                        op: CfgOp::And,
                    };
                }
                TokenType::Pipe => {
                    *pos += 1;
                    let right = self.parse_cfg_cmp(tokens, pos);
                    left = CfgExpr::Cmp {
                        left: Box::new(left),
                        right: Box::new(right),
                        op: CfgOp::Or,
                    };
                }
                _ => break,
            }
        }

        left
    }

    fn parse_cfg_cmp(&self, tokens: &[Token], pos: &mut usize) -> CfgExpr {
        let left = self.parse_cfg_atom(tokens, pos);

        if *pos < tokens.len() {
            match &tokens[*pos].token_type {
                TokenType::Equal => {
                    *pos += 1;
                    let right = self.parse_cfg_atom(tokens, pos);
                    CfgExpr::Cmp {
                        left: Box::new(left),
                        right: Box::new(right),
                        op: CfgOp::Eq,
                    }
                }
                TokenType::BangEqual => {
                    *pos += 1;
                    let right = self.parse_cfg_atom(tokens, pos);
                    CfgExpr::Cmp {
                        left: Box::new(left),
                        right: Box::new(right),
                        op: CfgOp::NotEq,
                    }
                }
                _ => left,
            }
        } else {
            left
        }
    }

    fn parse_cfg_atom(&self, tokens: &[Token], pos: &mut usize) -> CfgExpr {
        match &tokens[*pos].token_type {
            TokenType::StringLiteral(s) => {
                *pos += 1;
                CfgExpr::Value(s.clone())
            }
            TokenType::Identifier(s) => {
                *pos += 1;
                CfgExpr::Known(s.clone())
            }
            _ => panic!("unexpected token in @cfg: {:?}", tokens[*pos]),
        }
    }

    fn statement(
        &mut self,
        semi: bool,
        final_iter_in_loop: Option<Expr>,
    ) -> Result<Stmt, ParseError> {
        while self.match_token(&[TokenType::Newline]) {}

        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::At]) {
            let mut lookahead = self.current - 1;
            let mut found_def = false;
            while lookahead < self.tokens.len() {
                match &self.tokens[lookahead].token_type {
                    TokenType::At => {
                        lookahead += 1;
                        continue;
                    }
                    TokenType::Identifier(identifier) => {
                        match identifier.as_str() {
                            "variadic" | "trust_ret" | "inline" | "no_frame" => {
                                lookahead += 1;
                            }
                            _ => break,
                        }
                        continue;
                    }
                    TokenType::Newline => {
                        lookahead += 1;
                        continue;
                    }
                    TokenType::Def => {
                        found_def = true;
                        break;
                    }
                    _ => {
                        break;
                    }
                }
            }

            if found_def {
                self.current -= 1;
                return self.fn_dec();
            } else {
                return self.at_declaration();
            }
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement(false);
        }
        if self.match_token(&[TokenType::For]) {
            return self.while_statement(true);
        }
        if self.match_token(&[TokenType::Struct]) {
            return self.class_dec();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::Let]) {
            return self.var_dec();
        }
        if self.match_token(&[TokenType::Def]) {
            return self.fn_dec();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block(final_iter_in_loop)?));
        }
        if self.match_token(&[TokenType::Break]) {
            self.advance();
            return Ok(Stmt::Break);
        }
        if self.match_token(&[TokenType::Continue]) {
            self.advance();
            return Ok(Stmt::Continue);
        }

        let expr = self.expression()?;
        if semi {
            self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        }
        Ok(Stmt::Expression(expr))
    }

    fn var_dec(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier("".into()), "Expected variable name")?;
        let var_name = match &name.token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(name.clone())),
        };

        let mut var_type = Type::Inferred;

        if let TokenType::Colon = self.peek().token_type {
            self.advance();
            var_type = self.parse_type()?;
        }

        // self.consume(TokenType::Colon, "Expected ':' after variable name")?;

        self.consume(TokenType::Equal, "Expected '=' after variable type")?;

        let initializer = self.expression()?;

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Stmt::VarDecl {
            name: var_name,
            var_type,
            value: initializer,
        })
    }

    fn class_dec(&mut self) -> Result<Stmt, ParseError> {
        let name_tok = self.consume(TokenType::Identifier("".into()), "Expected class name")?;
        let class_name = if let TokenType::Identifier(n) = &name_tok.token_type {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken(name_tok.clone()));
        };

        let mut generics = vec![];

        if let TokenType::Less = self.peek().token_type {
            self.advance();
            if let TokenType::Identifier(generic) = &self.peek().token_type {
                generics.push(generic.clone());
                if !self.current_generics.insert(generic.to_string()) {
                    return Err(ParseError::Expected {
                        expected: TokenType::Identifier("generic parameter".to_string()),
                        found: self.peek().clone(),
                        message: "Duplicate generic params".to_owned(),
                    });
                }
            }

            self.advance();

            while self.peek().token_type == TokenType::Comma {
                self.advance();
                if let TokenType::Identifier(generic) = &self.peek().token_type {
                    generics.push(generic.clone());
                    if !self.current_generics.insert(generic.to_string()) {
                        return Err(ParseError::Expected {
                            expected: TokenType::Identifier("generic parameter".to_string()),
                            found: self.peek().clone(),
                            message: "Duplicate generic params".to_owned(),
                        });
                    }
                } else {
                    return Err(ParseError::Expected {
                        expected: TokenType::Identifier("generic parameter".to_owned()),
                        found: self.peek().clone(),
                        message: "Maybe eliminate trailing comma".to_owned(),
                    });
                }
                self.advance();
            }

            self.consume(
                TokenType::Greater,
                "expected '>' after struct's generic parameters",
            )?;
        }

        self.consume(TokenType::LeftBrace, "Expected '{' after class name")?;

        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let TokenType::Newline = self.peek().token_type {
                self.consume(TokenType::Newline, "unexpected error parsing struct fields")?;
                continue;
            }
            let field_name_tok =
                self.consume(TokenType::Identifier("".into()), "Expected field name")?;

            let field_name = if let TokenType::Identifier(n) = &field_name_tok.token_type {
                n.clone()
            } else {
                return Err(ParseError::UnexpectedToken(field_name_tok.clone()));
            };

            self.consume(TokenType::Colon, "Expected ':' after field name")?;
            let ty = self.parse_type()?;

            self.consume(TokenType::Semicolon, "Expected ';' after field declaration")?;
            fields.push((field_name, ty));
        }

        self.consume(TokenType::RightBrace, "Expected '}' after class body")?;

        self.current_generics = HashSet::new();

        Ok(Stmt::StructDecl {
            name: class_name,
            instances: fields,
            generics,
            union: false,
        })
    }

    fn fn_dec(&mut self) -> Result<Stmt, ParseError> {
        // Collect attributes before the function declaration
        let mut attributes = Vec::new();
        // println!("fn_dec: current token: {:?}", self.peek().token_type);
        while self.match_token(&[TokenType::At]) {
            // println!("Found @ token, next token: {:?}", self.peek().token_type);
            if let TokenType::Identifier(attr) = &self.peek().token_type.clone() {
                match attr.as_str() {
                    "variadic" | "trust_ret" | "inline" | "no_frame" => {}
                    _ => continue,
                };
                self.advance();
                attributes.push(attr.clone());
                // println!("Found attribute: {}", attr);

                while self.match_token(&[TokenType::Newline]) {}
            } else {
                return Err(ParseError::Expected {
                    expected: TokenType::Identifier("attribute".to_string()),
                    found: self.peek().clone(),
                    message: "Expected attribute name after '@'".to_string(),
                });
            }
        }
        // println!("Collected attributes: {:?}", attributes);

        // Skip newlines after attributes
        while self.match_token(&[TokenType::Newline]) {
            // Just consume the newline token
        }

        // Consume the 'def' keyword if we have attributes (it was already consumed by the lookahead)
        if !attributes.is_empty() {
            self.consume(TokenType::Def, "Expected 'def' keyword")?;
        }

        let name = self.consume(TokenType::Identifier("".into()), "Expected function name")?;
        let fun_name = match &name.token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => return Err(ParseError::UnexpectedToken(name.clone())),
        };

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name =
                    self.consume(TokenType::Identifier("".into()), "Expected parameter name")?;
                let param_name_str = match &param_name.token_type {
                    TokenType::Identifier(n) => n.clone(),
                    _ => return Err(ParseError::UnexpectedToken(param_name.clone())),
                };

                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;
                parameters.push((param_name_str, param_type));

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        if let TokenType::LeftBrace = self.peek().token_type {
            self.advance();
            let body = self.block(None)?;

            return Ok(Stmt::FunDecl {
                name: fun_name,
                params: parameters,
                return_type: Type::Void,
                body,
                attributes,
            });
        }

        self.consume(TokenType::DoubleColon, "Expected '::' after parameters")?;
        let return_type = self.parse_type()?;
        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.block(None)?;

        Ok(Stmt::FunDecl {
            name: fun_name,
            params: parameters,
            return_type,
            body,
            attributes,
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        let then_branch = Box::new(self.statement(true, None)?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement(true, None)?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_stmt: then_branch,
            else_stmt: else_branch,
        })
    }

    fn while_statement(&mut self, is_for: bool) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        if is_for {
            if let TokenType::DoubleColon = self.peek().token_type {
                self.advance();
                let iter = self.expression()?;
                match iter {
                    Expr::Assign { .. }
                    | Expr::DerefAssign { .. }
                    | Expr::CompoundAssign { .. }
                    | Expr::PreIncrement { .. }
                    | Expr::PostIncrement { .. }
                    | Expr::PreDecrement { .. }
                    | Expr::PostDecrement { .. } => {
                        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
                        let body = Box::new(self.statement(true, Some(iter))?);

                        return Ok(Stmt::While { condition, body });
                    }
                    _ => return Err(ParseError::UnexpectedToken(self.peek().clone())),
                }
            }
            Err(ParseError::Expected {
                expected: TokenType::DoubleColon,
                found: self.peek().clone(),
                message: "Expected increment".to_owned(),
            })
        } else {
            self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
            let body = Box::new(self.statement(true, None)?);

            Ok(Stmt::While { condition, body })
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logic_or()?;

        if self.match_token(&[
            TokenType::Equal,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
        ]) {
            let op_token = self.previous().token_type.clone();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name, _) => match op_token {
                    TokenType::Equal => {
                        return Ok(Expr::Assign {
                            name,
                            value: Box::new(value),
                        });
                    }
                    TokenType::PlusEqual => {
                        return Ok(Expr::CompoundAssign {
                            name,
                            op: BinaryOp::Add,
                            value: Box::new(value),
                        });
                    }
                    TokenType::MinusEqual => {
                        return Ok(Expr::CompoundAssign {
                            name,
                            op: BinaryOp::Sub,
                            value: Box::new(value),
                        });
                    }
                    TokenType::StarEqual => {
                        return Ok(Expr::CompoundAssign {
                            name,
                            op: BinaryOp::Mul,
                            value: Box::new(value),
                        });
                    }
                    TokenType::SlashEqual => {
                        return Ok(Expr::CompoundAssign {
                            name,
                            op: BinaryOp::Div,
                            value: Box::new(value),
                        });
                    }
                    _ => unreachable!(),
                },

                Expr::Unary {
                    op: UnaryOp::Dereference,
                    ref expr,
                    ..
                } => {
                    return Ok(Expr::DerefAssign {
                        target: expr.clone(),
                        value: Box::new(value),
                    });
                }
                Expr::InstanceVar(class_name, instance_name) => {
                    // return Ok(Expr::InstanceVar(class_name, instance_name));

                    return Ok(Expr::FieldAssign {
                        class_name,
                        field: instance_name,
                        value: Box::new(value),
                    });

                    // return Ok(Expr::DerefAssign {
                    //     target: Box::new(Expr::InstanceVar(class_name, instance_name)),
                    //     value: Box::new(value),
                    // });
                }
                Expr::ArrayAccess { array, index } => {
                    // return Ok( Expr::DerefAssign {
                    //    target: Box::new(Expr::ArrayAccess { array, index }),
                    //    value: Box::new(value),
                    // });

                    return Ok(Expr::IndexAssign {
                        array,
                        index,
                        value: Box::new(value),
                    });

                    // return Ok(Expr::ArrayAccess { array, index })
                }

                _ => return Err(ParseError::InvalidAssignmentTarget),
            }
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while self.match_token(&[TokenType::Or]) {
            let right = self.logic_and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.previous().token_type {
                TokenType::BangEqual => BinaryOp::NotEqual,
                TokenType::EqualEqual => BinaryOp::Equal,
                _ => unreachable!(),
            };

            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = match self.previous().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };

            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let op = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Sub,
                TokenType::Plus => BinaryOp::Add,
                _ => unreachable!(),
            };
            // if let TokenType::Plus = self.peek().token_type {
            //     self.advance();
            //     expr = Expr::Binary {
            //         left: Box::new(expr),
            //         op,
            //         right: Box::new(Expr::IntLiteral(1)),
            //         result_type: Type::Unknown,
            //     };
            // } else if let TokenType::Minus = self.peek().token_type {
            //     self.advance();
            //     expr = Expr::Binary {
            //         left: Box::new(expr),
            //         op,
            //         right: Box::new(Expr::IntLiteral(1)),
            //         result_type: Type::Unknown,
            //     };
            // } else {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
            // }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let op = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Div,
                TokenType::Star => BinaryOp::Mul,
                TokenType::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };

            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    // fn unary(&mut self) -> Result<Expr, ParseError> {
    //     if self.match_token(&[
    //         TokenType::Bang,
    //         TokenType::Minus,
    //         TokenType::Star,
    //         TokenType::Ampersand,
    //     ]) {
    //         let op = match self.previous().token_type {
    //             TokenType::Bang => UnaryOp::Not,
    //             TokenType::Minus => UnaryOp::Negate,
    //             TokenType::Star => UnaryOp::Dereference,
    //             TokenType::Ampersand => UnaryOp::AddressOf,
    //             _ => unreachable!(),
    //         };

    //         let right = self.unary()?;
    //         return Ok(Expr::Unary {
    //             op,
    //             expr: Box::new(right.clone()),
    //             result_type: Type::Unknown,
    //         });
    //     }

    //     self.cast()
    // }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[
            TokenType::Bang,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Ampersand,
            TokenType::PlusPlus,
            TokenType::MinusMinus,
        ]) {
            let op_token = self.previous().token_type.clone();

            // Handle prefix increment/decrement
            if matches!(op_token, TokenType::PlusPlus | TokenType::MinusMinus) {
                let expr = self.unary()?;
                if let Expr::Variable(name, _) = expr {
                    return Ok(match op_token {
                        TokenType::PlusPlus => Expr::PreIncrement { name },
                        TokenType::MinusMinus => Expr::PreDecrement { name },
                        _ => unreachable!(),
                    });
                } else {
                    return Err(ParseError::InvalidAssignmentTarget);
                }
            }

            let op = match op_token {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Negate,
                TokenType::Star => UnaryOp::Dereference,
                TokenType::Ampersand => UnaryOp::AddressOf,
                _ => unreachable!(),
            };

            let right = self.unary()?;
            let right_type = right.get_type();

            let result_type = match op {
                UnaryOp::Dereference => match right_type {
                    Type::Pointer(pointee_type) => *pointee_type,
                    Type::Void => Type::Void,
                    _ => {
                        // return Err(ParseError::InvalidAssignmentTarget);

                        return Ok(Expr::Unary {
                            op,
                            expr: Box::new(right),
                            result_type: Type::Unknown,
                        });
                    }
                },
                UnaryOp::AddressOf => Type::Pointer(Box::new(right_type)),
                UnaryOp::Not => Type::Bool,
                UnaryOp::Negate => right_type,
            };

            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
                result_type,
            });
        }

        self.cast()
    }

    fn cast(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.call()?;

        if self.match_token(&[TokenType::As]) {
            let target_type = self.parse_type()?;
            expr = Expr::Cast {
                expr: Box::new(expr),
                target_type,
            };
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::PlusPlus, TokenType::MinusMinus]) {
                let op_token = self.previous().token_type.clone();
                if let Expr::Variable(name, _) = expr {
                    expr = match op_token {
                        TokenType::PlusPlus => Expr::PostIncrement { name },
                        TokenType::MinusMinus => Expr::PostDecrement { name },
                        _ => unreachable!(),
                    };
                } else {
                    return Err(ParseError::InvalidAssignmentTarget);
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        if let Expr::Variable(name, ty) = callee {
            Ok(Expr::Call {
                name,
                args: arguments,
                return_type: ty,
            })
        } else {
            Err(ParseError::InvalidCallTarget)
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match &self.peek().token_type {
            TokenType::Newline => {
                self.advance();
                while self.match_token(&[TokenType::Newline]) {}
                self.expression()
            }
            TokenType::StringLiteral(str) => {
                let str = str.clone();
                self.advance();
                Ok(Expr::StringLiteral(str))
            }
            TokenType::IntLiteral(val) => {
                let val = *val;
                self.advance();
                Ok(Expr::IntLiteral(val))
            }
            TokenType::FloatLiteral(val) => {
                let val = *val;
                self.advance();
                Ok(Expr::FloatLiteral(val))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            TokenType::CharLiteral(c) => {
                let c = *c;
                self.advance();
                Ok(Expr::CharLiteral(c))
            }
            TokenType::DoubleQuote => {
                self.advance();
                if let TokenType::Identifier(n) = &self.peek().clone().token_type {
                    self.consume(
                        TokenType::DoubleColon,
                        "Expected double quote to end string",
                    )?;
                    return Ok(Expr::StringLiteral(n.to_string()));
                }
                Err(ParseError::Expected {
                    expected: TokenType::Identifier("".to_string()),
                    found: self.peek().clone(),
                    message: "Expected double quote".to_owned(),
                })
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();

                if !self.check(&TokenType::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RightBracket, "Expected ']' after array elements")?;

                let element_type = elements.first().map_or(Type::Unknown, |e| match e {
                    Expr::IntLiteral(_) => Type::int,
                    Expr::FloatLiteral(_) => Type::float,
                    Expr::BoolLiteral(_) => Type::Bool,
                    Expr::CharLiteral(_) => Type::Char,
                    // Expr::Array(exprs, ty) => Type::Array(
                    //     Box::new(Type::Array(Box::new(exprs[0].get_type(), None))),
                    //     Some(exprs.len()),
                    // ),
                    _ => Type::Unknown,
                });

                Ok(Expr::Array(elements, element_type))
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.peek().token_type == TokenType::LeftBrace {
                    let mut inits: Vec<(String, Expr)> = Vec::new();
                    self.advance();

                    // self.consume(TokenType::LeftBrace, "Expected '(' to introduce class init")?;

                    if !self.check(&TokenType::RightBrace) {
                        loop {
                            if let TokenType::Newline = self.peek().token_type {
                                self.advance();
                            }

                            let fname_tok = self
                                .consume(TokenType::Identifier("".into()), "Expected field name")?;
                            let fname = if let TokenType::Identifier(n) = &fname_tok.token_type {
                                n.clone()
                            } else {
                                return Err(ParseError::UnexpectedToken(fname_tok.clone()));
                            };

                            self.consume(TokenType::Colon, "Expected ':' after field name")?;
                            let val = self.expression()?;
                            inits.push((fname, val));

                            if !self.match_token(&[TokenType::Comma]) {
                                break;
                            }
                        }
                    }

                    if let TokenType::Newline = self.peek().token_type {
                        self.advance();
                    }

                    self.consume(
                        TokenType::RightBrace,
                        "Expected ')' after class initializer",
                    )?;

                    return Ok(Expr::StructInit {
                        name,
                        params: inits,
                    });
                }

                if self.peek().token_type == TokenType::LeftBracket {
                    let array = Box::new(Expr::Variable(name.clone(), Type::Unknown));
                    self.advance();
                    let peeked = self.peek().clone().token_type;
                    match peeked {
                        TokenType::IntLiteral(index) => {
                            self.advance();

                            self.consume(
                                TokenType::RightBracket,
                                "Right bracket expected for array indexing",
                            )?;

                            return Ok(Expr::ArrayAccess {
                                array,
                                index: Box::new(Expr::IntLiteral(index)),
                            });
                        }
                        TokenType::Identifier(name) => {
                            self.advance();

                            self.consume(
                                TokenType::RightBracket,
                                "Right bracket expected for array indexing",
                            )?;

                            return Ok(Expr::ArrayAccess {
                                array,
                                index: Box::new(Expr::Variable(name.to_string(), Type::Unknown)),
                            });
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken(self.peek().clone()));
                        }
                    }
                }

                if self.peek().token_type == TokenType::Period {
                    self.advance();
                    if let TokenType::Identifier(var_name) = &self.peek().token_type.clone() {
                        self.consume(
                            TokenType::Identifier(var_name.to_string()),
                            "Could not find instance",
                        )?;
                        return Ok(Expr::InstanceVar(name, var_name.to_string()));
                    }
                }

                Ok(Expr::Variable(name.clone(), Type::Unknown))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken(self.peek().clone())),
        }
    }

    fn _peek_next(&mut self) -> &Token {
        if self.current + 1 >= self.tokens.len() {
            &self.tokens[self.current]
        } else {
            &self.tokens[self.current + 1]
        }
    }

    fn block(&mut self, last_expr: Option<Expr>) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines before statements
            while self.match_token(&[TokenType::Newline]) {
                // Just consume the newline token
            }

            if !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                statements.push(self.statement(true, None)?);
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        if let Some(last_expr) = last_expr {
            statements.push(Stmt::Expression(last_expr));
        }
        Ok(statements)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&[TokenType::LeftBracket]) {
            let elem = self.parse_type()?;

            if self.match_token(&[TokenType::RightBracket]) {
                return Ok(Type::Array(Box::new(elem), None));
            }

            self.consume(
                TokenType::Comma,
                "Expected ',' after element type or ']' for slice",
            )?;

            let size_tok = self.consume(TokenType::IntLiteral(0), "Expected array size")?;
            let size = match size_tok.token_type {
                TokenType::IntLiteral(n) => n as usize,
                _ => return Err(ParseError::UnexpectedToken(size_tok.clone())),
            };
            self.consume(TokenType::RightBracket, "Expected ']' after array size")?;
            return Ok(Type::Array(Box::new(elem), Some(size)));
        }

        let token_type = &self.peek().token_type;
        let mut base_type = match token_type {
            TokenType::Int => {
                self.advance();
                Type::int
            }
            TokenType::Float => {
                self.advance();
                Type::float
            }
            TokenType::Boolean => {
                self.advance();
                Type::Bool
            }
            TokenType::Void => {
                self.advance();
                Type::Void
            }
            TokenType::Char => {
                self.advance();
                Type::Char
            }
            TokenType::Long => {
                self.advance();
                Type::Long
            }
            TokenType::Identifier(name) => {
                let struct_name = name.clone();

                if self.current_generics.contains(&struct_name) {
                    self.advance();
                    Type::Generic(struct_name)
                } else {
                    self.advance();
                    
                    let mut generics = vec![];
                    if let TokenType::Less = self.peek().token_type {
                        self.advance(); // consume '<'
                        generics.push(self.parse_type()?);
                        while self.peek().token_type == TokenType::Comma {
                            self.advance(); // consume ','
                            generics.push(self.parse_type()?);
                        }
                        self.consume(
                            TokenType::Greater,
                            "expected '>' after generic type arguments",
                        )?;
                    }
                    // Type::Pointer(Box::new(
                    Type::Struct {
                        name: struct_name,
                        instances: Vec::new(),
                        generics,
                    }
                }
                // ))
            }
            _ => return Err(ParseError::UnexpectedToken(self.peek().clone())),
        };

        while self.match_token(&[TokenType::Star]) {
            base_type = Type::Pointer(Box::new(base_type));
        }

        Ok(base_type)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Expected {
        expected: TokenType,
        found: Token,
        message: String,
    },
    UnexpectedToken(Token),
    InvalidCallTarget,
    InvalidAssignmentTarget,
}
