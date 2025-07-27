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
            statements.push(self.statement()?);
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

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[TokenType::Var]) {
            return self.var_dec();
        }
        if self.match_token(&[TokenType::Fn]) {
            return self.fn_dec();
        }
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        self.expression_statement()
    }

    fn var_dec(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenType::Identifier("".to_string()),
            "Expected variable name",
        )?;
        let var_name = if let TokenType::Identifier(n) = &name.token_type {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken(name.clone()));
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

    fn fn_dec(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(
            TokenType::Identifier("".to_string()),
            "Expected a function name",
        )?;
        let fun_name = if let TokenType::Identifier(n) = &name.token_type {
            n.clone()
        } else {
            return Err(ParseError::UnexpectedToken(name.clone()));
        };

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;

        let mut parameters = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                let param_name = self.consume(
                    TokenType::Identifier("".to_string()),
                    "expected a parameter name",
                )?;
                let param_name_str = if let TokenType::Identifier(n) = &param_name.token_type {
                    n.clone()
                } else {
                    return Err(ParseError::UnexpectedToken(param_name.clone()));
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
        self.consume(TokenType::Arrow, "Expected '->' after parameters")?;

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

        self.consume(TokenType::RightParen, "Expected ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
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
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logic_or()?;

        if self.match_token(&[TokenType::Equal]) {
            let _equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            };

            return Err(ParseError::InvalidAssignmentTarget);
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while self.match_token(&[TokenType::Or]) {
            let _operator = self.previous().clone();
            let right = self.logic_and()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
                result_type: Type::Unknown,
            }
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let _operator = self.previous().clone();

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
            let operator = self.previous().clone();

            let right = self.comparison()?;

            let op = match operator.token_type {
                TokenType::BangEqual => BinaryOp::NotEqual,
                TokenType::EqualEqual => BinaryOp::Equal,
                _ => unreachable!(),
            };

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
            let operator = self.previous().clone();

            let right = self.term()?;

            let op = match operator.token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;

            let op = match operator.token_type {
                TokenType::Minus => BinaryOp::Sub,
                TokenType::Plus => BinaryOp::Add,
                _ => unreachable!(),
            };

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            let op = match operator.token_type {
                TokenType::Slash => BinaryOp::Div,
                TokenType::Star => BinaryOp::Mul,
                _ => unreachable!(),
            };

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                result_type: Type::Unknown,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            let op = match operator.token_type {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Negate,
                _ => unreachable!(),
            };

            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
                result_type: Type::Unknown,
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
            }
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

        if let Expr::Variable(name) = callee {
            Ok(Expr::Call {
                name,
                args: arguments,
                return_type: Type::Unknown,
            })
        } else {
            Err(ParseError::InvalidCallTarget)
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if let TokenType::IntLiteral(val) = &self.peek().token_type {
            let val = *val;

            self.advance();

            return Ok(Expr::IntLiteral(val.into()));
        }

        if let TokenType::FloatLiteral(val) = &self.peek().token_type {
            let val = *val;

            self.advance();

            return Ok(Expr::FloatLiteral(val.into()));
        }

        if self.match_token(&[TokenType::LeftBracket]) {
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

            // Try to infer the array type from elements
            let element_type = if !elements.is_empty() {
                match &elements[0] {
                    Expr::IntLiteral(_) => Type::int,
                    Expr::FloatLiteral(_) => Type::float,
                    Expr::BoolLiteral(_) => Type::Bool,
                    Expr::CharLiteral(_) => Type::Char,
                    Expr::Array(_, ty) => ty.clone(),
                    _ => Type::Unknown,
                }
            } else {
                Type::Unknown
            };

            return Ok(Expr::Array(elements, element_type));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::BoolLiteral(true));
        }

        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::BoolLiteral(false));
        }

        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            return Ok(Expr::Variable(name));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;

            return Ok(expr);
        }

        Err(ParseError::UnexpectedToken(self.peek().clone()))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) {
            statements.push(self.statement()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;

        Ok(statements)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.match_token(&[TokenType::LeftBracket]) {
            // Parse element type
            let element_type = self.parse_type()?;

            self.consume(TokenType::Comma, "Expected ',' after array element type")?;

            // Parse array size
            let size_token = self.consume(TokenType::IntLiteral(0), "Expected array size")?;
            let size = if let TokenType::IntLiteral(size) = size_token.token_type {
                size as usize
            } else {
                return Err(ParseError::UnexpectedToken(size_token.clone()));
            };

            self.consume(TokenType::RightBracket, "Expected ']' after array size")?;

            Ok(Type::Array(Box::new(element_type), size))
        } else {
            // Handle base types
            match &self.peek().token_type {
                TokenType::Int => {
                    self.advance();
                    Ok(Type::int)
                }
                TokenType::Float => {
                    self.advance();
                    Ok(Type::float)
                }
                TokenType::Boolean => {
                    self.advance();
                    Ok(Type::Bool)
                }
                TokenType::Void => {
                    self.advance();
                    Ok(Type::Void)
                }
                TokenType::Char => {
                    self.advance();
                    Ok(Type::Char)
                }
                _ => Err(ParseError::UnexpectedToken(self.peek().clone())),
            }
        }
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
