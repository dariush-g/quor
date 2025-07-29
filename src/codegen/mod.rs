use crate::lexer::ast::{BinaryOp, Expr, Stmt, Type};
use std::collections::VecDeque;

pub struct CodeGen {
    output: String,
    regs: VecDeque<String>,
    used: VecDeque<(String, String)>,
    _jmp_count: u32,
    functions: Vec<(String, Vec<(String, Type)>, Vec<Stmt>)>,
}

impl CodeGen {
    pub fn generate(stmts: &Vec<Stmt>) -> String {
        let mut code = CodeGen {
            output: String::new(),
            regs: VecDeque::from(vec![
                "rax".to_string(),
                "rcx".to_string(),
                "rdx".to_string(),
                "rsi".to_string(),
                "rdi".to_string(),
                "r10".to_string(),
                "r11".to_string(),
            ]),
            used: VecDeque::new(),
            _jmp_count: 0,
            functions: vec![],
        };

        let mut has_main = false;

        for stmt in stmts {
            if let Stmt::FunDecl {
                name,
                params,
                return_type: _,
                body,
            } = stmt
            {
                if name == "main" {
                    code.generate_function("_start", &vec![], body);
                    has_main = true;
                } else {
                    code.functions
                        .push((name.clone(), params.clone(), body.clone()));
                }
            }
        }

        if !has_main {
            panic!("No main function found");
        }

        let functions = &code.functions.clone();

        for (name, params, body) in functions {
            code.generate_function(name, params, body);
        }

        for stmt in stmts {
            if !matches!(stmt, Stmt::FunDecl { .. }) {
                code.handle_stmt(stmt);
            }
        }

        code.output
    }

    fn handle_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                var_type: _,
                value,
            } => {
                self.handle_expr(value, Some(name.clone()));
            }
            Stmt::Expression(expr) => match expr {
                Expr::Call {
                    name,
                    args,
                    return_type,
                } => {
                    // TODO: allow parameters through registers bf fn call
                    self.output.push_str(&format!("call {}\n", name));
                }
                _ => {}
            },
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => match condition {
                Expr::BoolLiteral(bool) => {}
                Expr::Variable(string) => {
                    
                }
                Expr::Binary {
                    left,
                    op,
                    right,
                    result_type,
                } => todo!(),
                Expr::Unary {
                    op,
                    expr,
                    result_type,
                } => todo!(),
                _ => {}
            },
            _ => {}
        }
    }

    fn generate_function(&mut self, name: &str, _: &Vec<(String, Type)>, body: &Vec<Stmt>) {
        self.output
            .push_str(&format!("global {}\n{}:\n", name, name));
        self.output.push_str("push rbp\n");
        self.output.push_str("mov rbp, rsp\n");

        for stmt in body {
            self.handle_stmt(stmt);
        }

        self.output.push_str("pop rbp\n");

        if name == "_start" {
            self.output
                .push_str(&format!("mov rax, 60\nxor rdi, rdi\nsyscall\n"));
        } else {
            self.output.push_str("ret\n");
        }
    }

    fn handle_expr(&mut self, expr: &Expr, ident: Option<String>) -> Option<String> {
        match expr {
            Expr::Call {
                name,
                args,
                return_type: _,
            } => {}
            Expr::IntLiteral(n) => {
                let av_reg = self.regs.pop_front().unwrap();
                self.output.push_str(&format!("mov {}, {n}\n", av_reg));
                if let Some(id) = ident {
                    self.used.push_back((av_reg.clone(), id));
                }
                return Some(av_reg.to_string());
            }
            Expr::AddressOf(expr) => {}
            Expr::Variable(name) => {
                let val = self.used.iter().find(|x| x.1 == name.to_string()).unwrap();
                return Some(val.0.to_string());
            }
            Expr::Assign { name, value: _ } => {}
            Expr::Binary {
                left,
                op,
                right,
                result_type: _,
            } => {
                let lhs = self.handle_expr(left, None).unwrap();
                let rhs = self.handle_expr(right, None).unwrap();
                match op {
                    BinaryOp::Add { .. } => {
                        let av_reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {}, {}\n", av_reg, lhs));
                        self.output.push_str(&format!("add {}, {}\n", av_reg, rhs));
                        if let Some(id) = ident {
                            self.used.push_back((av_reg.clone(), id));
                        }
                        return Some(av_reg.to_owned());
                    }
                    BinaryOp::Sub { .. } => {
                        let av_reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {}, {}\n", av_reg, lhs));
                        self.output.push_str(&format!("sub {}, {}\n", av_reg, rhs));
                        if let Some(id) = ident {
                            self.used.push_back((av_reg.clone(), id));
                        }
                        return Some(av_reg.to_owned());
                    }
                    BinaryOp::Mul { .. } => {
                        let av_reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {}, {}\n", av_reg, lhs));
                        self.output.push_str(&format!("imul {}, {}\n", av_reg, rhs));
                        if let Some(id) = ident {
                            self.used.push_back((av_reg.clone(), id));
                        }
                        return Some(av_reg.to_owned());
                    }
                    BinaryOp::Div { .. } => {
                        let rax_index = self.used.iter().position(|(reg, _)| *reg == "rax");

                        if let Some(idx) = rax_index {
                            let (_, old_name) = self.used.remove(idx).unwrap();

                            let tmp_reg = self
                                .regs
                                .pop_front()
                                .expect("No free registers to save rax");
                            self.output.push_str(&format!("mov {}, rax\n", tmp_reg));
                            self.used.push_back((tmp_reg, old_name));
                        }

                        self.output.push_str(&format!("mov rax, {}\n", lhs));
                        self.output.push_str("xor rdx, rdx\n");
                        self.output.push_str(&format!("div {}\n", rhs));

                        let result_reg = self.regs.pop_front().expect("No free registers");
                        self.output.push_str(&format!("mov {}, rax\n", result_reg));

                        if let Some(id) = ident {
                            self.used.push_back((result_reg.clone(), id));
                        }

                        return Some(result_reg.to_owned());
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        None
    }
}
