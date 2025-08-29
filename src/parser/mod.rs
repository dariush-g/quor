use crate::lexer::{
    ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
    token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement(true)?);
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

            if let TokenType::DoubleColon = self.peek().token_type {
                self.advance();
                if let TokenType::Less = self.peek().token_type {
                    self.advance(); // consume '('
                    let mut param = if let TokenType::Identifier(s) = &self.peek().token_type {
                        let val = s.clone();
                        self.advance(); // consume string
                        val
                    } else {
                        return Err(ParseError::Expected {
                            expected: TokenType::Identifier("".to_string()),
                            found: self.peek().clone(),
                            message: "Expected identifier in @import".to_owned(),
                        });
                    };

                    if let TokenType::Period = self.peek().token_type {
                        param.push('.');
                        self.advance();
                        if let TokenType::Identifier(qu) = &self.peek().token_type {
                            param.push_str(&qu);
                            self.advance();
                        }
                    }

                    param.push('!');

                    self.consume(TokenType::Greater, "Expected '>'")?;
                    self.consume(TokenType::Semicolon, "Expected ';'")?;
                    return Ok(Stmt::AtDecl(decl.to_string(), Some(param)));
                }
                if let TokenType::LeftParen = self.peek().token_type {
                    self.advance(); // consume '('
                    let mut param = if let TokenType::Identifier(s) = &self.peek().token_type {
                        let val = s.clone();
                        self.advance(); // consume string
                        val
                    } else {
                        return Err(ParseError::Expected {
                            expected: TokenType::Identifier("".to_string()),
                            found: self.peek().clone(),
                            message: "Expected identifier in @import".to_owned(),
                        });
                    };

                    if let TokenType::Period = self.peek().token_type {
                        param.push('.');
                        self.advance();
                        if let TokenType::Identifier(qu) = &self.peek().token_type {
                            param.push_str(&qu);
                            self.advance();
                        }
                    }

                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    self.consume(TokenType::Semicolon, "Expected ';'")?;

                    return Ok(Stmt::AtDecl(decl.to_string(), Some(param)));
                }
            }

            return Ok(Stmt::AtDecl(decl.to_string(), None));
        }

        Err(ParseError::Expected {
            expected: TokenType::Identifier("declaration".to_string()),
            found: self.peek().clone(),
            message: "Expected declaration after '@'".to_owned(),
        })
    }

    fn statement(&mut self, semi: bool) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::At]) {
            return self.at_declaration();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
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
            return Ok(Stmt::Block(self.block()?));
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

        self.consume(TokenType::Colon, "Expected ':' after variable name")?;

        let var_type = self.parse_type()?;

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

        self.consume(TokenType::LeftBrace, "Expected '{' after class name")?;

        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
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

        Ok(Stmt::StructDecl {
            name: class_name,
            instances: fields,
        })
    }

    fn fn_dec(&mut self) -> Result<Stmt, ParseError> {
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
        self.consume(TokenType::DoubleColon, "Expected '::' after parameters")?;
        let return_type = self.parse_type()?;
        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.block()?;

        Ok(Stmt::FunDecl {
            name: fun_name,
            params: parameters,
            return_type,
            body,
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        let then_branch = Box::new(self.statement(false)?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement(false)?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_stmt: then_branch,
            else_stmt: else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
        let body = Box::new(self.statement(false)?);

        Ok(Stmt::While { condition, body })
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

        if self.match_token(&[TokenType::Equal]) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name, _) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }

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

            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let op = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Div,
                TokenType::Star => BinaryOp::Mul,
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
        ]) {
            let op = match self.previous().token_type {
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

        while self.match_token(&[TokenType::LeftParen]) {
            expr = self.finish_call(expr)?;
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

                if self.peek().token_type == TokenType::DoubleColon {
                    let mut inits: Vec<(String, Expr)> = Vec::new();
                    self.advance();

                    self.consume(TokenType::LeftParen, "Expected '(' to introduce class init")?;

                    if !self.check(&TokenType::RightParen) {
                        loop {
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

                    self.consume(
                        TokenType::RightParen,
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

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.statement(true)?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
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
                // Handle class names as types
                let struct_name = name.clone();
                self.advance();
                // Type::Pointer(Box::new(
                Type::Struct {
                    name: struct_name,
                    instances: Vec::new(),
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
