use crate::lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp};
use std::collections::{HashMap, HashSet, VecDeque};

type Functions = Vec<(String, Vec<(String, Type)>, Vec<Stmt>)>;

pub struct CodeGen {
    output: String,
    regs: VecDeque<String>,
    _jmp_count: u32,
    functions: Functions,
    locals: HashMap<String, i32>,
    stack_size: i32,
    externs: HashSet<String>,
}

impl CodeGen {
    pub fn generate(stmts: &Vec<Stmt>) -> String {
        let mut code = CodeGen {
            output: String::new(),
            regs: VecDeque::from(vec![
                "rcx".to_string(),
                "rdx".to_string(),
                "rsi".to_string(),
                "rdi".to_string(),
                "r10".to_string(),
                "r11".to_string(),
            ]),
            _jmp_count: 0,
            functions: vec![],
            locals: HashMap::new(),
            stack_size: 0,
            externs: HashSet::new(),
        };

        code.output.push_str("global _start\n_start:\n");
        code.output.push_str("call main\n"); // main(): returns int in rax
        code.output.push_str("mov rbx, rax\n"); // keep a copy in RBX
        // macOS x86_64 exit(status): rax=0x2000001, rdi=status
        code.output.push_str("mov rdi, rax\n");
        code.output.push_str("mov rax, 0x2000001\n");
        code.output.push_str("syscall\n");
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
                    code.generate_function("main", &vec![], body);
                    has_main = true;

                    // code.output
                    //     .push_str("mov rax, 0x2000001\nxor rdi, rdi\nsyscall\n");
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

        let defined: HashSet<String> = code.functions.iter().map(|(n, _, _)| n.clone()).collect();

        let mut header = String::new();
        for ext in code.externs.difference(&defined) {
            header.push_str(&format!("extern _{ext}\n"));
        }

        format!("{header}{}", code.output)
    }

    fn alloc_local(&mut self, name: &str, _ty: &Type) -> i32 {
        // every local is 8 bytes
        self.stack_size += 8;
        self.output.push_str("sub rsp, 8\n");
        let offset = self.stack_size; // addressable as [rbp - offset]
        self.locals.insert(name.to_string(), offset);
        offset
    }

    fn local_offset(&self, name: &str) -> Option<i32> {
        self.locals.get(name).cloned()
    }

    fn handle_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => {
                match value {
                    Expr::Array(elements, _ty) => {
                        let count = elements.len();
                        let total_size = (count as i32) * 8; // assuming 8 bytes per slot
                        self.stack_size += total_size;
                        self.output.push_str(&format!("sub rsp, {total_size}\n"));

                        let base_offset = self.stack_size;
                        self.locals.insert(name.clone(), base_offset);

                        for (i, element) in elements.iter().enumerate() {
                            let offset = base_offset - (i as i32) * 8;

                            match element {
                                Expr::IntLiteral(n) => {
                                    self.output
                                        .push_str(&format!("mov QWORD [rbp - {offset}], {n}\n"));
                                }
                                Expr::BoolLiteral(b) => {
                                    let val = if *b { 1 } else { 0 };
                                    self.output
                                        .push_str(&format!("mov QWORD [rbp - {offset}], {val}\n"));
                                }
                                _ => {
                                    if let Some(val_reg) = self.handle_expr(element, None) {
                                        self.output.push_str(&format!(
                                            "mov QWORD [rbp - {offset}], {val_reg}\n"
                                        ));
                                        self.regs.push_back(val_reg);
                                    }
                                }
                            }
                        }
                    }

                    _ => {
                        // fall back to scalar variable allocation
                        let offset = if self.local_offset(name).is_none() {
                            self.alloc_local(name, var_type)
                        } else {
                            self.local_offset(name).unwrap()
                        };

                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov QWORD [rbp - {offset}], {val_reg}\n"));
                            self.regs.push_back(val_reg);
                        }
                    }
                }
            }

            Stmt::Expression(expr) => {
                if let Some(reg) = self.handle_expr(expr, None) {
                    // expression result unused; free temp register
                    self.regs.push_back(reg);
                }
            }
            Stmt::Return(expr) =>
            {
                #[allow(clippy::collapsible_match)]
                if let Some(ex) = expr {
                    self.handle_expr(ex, None);
                }
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                // Evaluate condition
                let cond_reg = self
                    .handle_expr(condition, None)
                    .expect("if condition should produce a register");
                let jmp_id = self._jmp_count;
                self._jmp_count += 1;

                // Compare to 0, jump if equal (false)
                self.output
                    .push_str(&format!("cmp {cond_reg}, 0\nje .else{jmp_id}\n"));
                // done with cond_reg
                self.regs.push_back(cond_reg);

                // then branch
                self.handle_stmt(then_stmt);
                if else_stmt.is_some() {
                    self.output.push_str(&format!("jmp .endif{jmp_id}\n"));
                }

                // else label
                self.output.push_str(&format!(".else{jmp_id}:\n"));
                if let Some(else_s) = else_stmt {
                    self.handle_stmt(else_s);
                    self.output.push_str(&format!(".endif{jmp_id}:\n"));
                }
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.handle_stmt(stmt);
                }
            }
            _ => {}
        }
    }

    fn generate_function(&mut self, name: &str, _: &Vec<(String, Type)>, body: &Vec<Stmt>) {
        self.locals.clear();
        self.stack_size = 0;

        let epilogue = format!(".Lret_{name}");

        self.output.push_str(&format!("global {name}\n{name}:\n"));
        self.output.push_str("push rbp\nmov rbp, rsp\n");

        for stmt in body {
            self.handle_stmt_with_epilogue(stmt, &epilogue);
        }

        if name == "main" {
            self.output.push_str("xor rax, rax\n");
        }

        self.output.push_str(&format!("{epilogue}:\n"));
        self.output.push_str("mov rsp, rbp\npop rbp\nret\n");
    }

    fn handle_stmt_with_epilogue(&mut self, stmt: &Stmt, epilogue: &str) {
        match stmt {
            Stmt::Return(opt) => {
                if let Some(ex) = opt {
                    if let Some(reg) = self.handle_expr(ex, None) {
                        if reg != "rax" {
                            self.output.push_str(&format!("mov rax, {reg}\n"));
                            self.regs.push_back(reg);
                        }
                    } else {
                        self.output.push_str("xor rax, rax\n");
                    }
                }

                self.output.push_str(&format!("jmp {epilogue}\n"));
            }
            _ => self.handle_stmt(stmt),
        }
    }

    fn handle_expr(&mut self, expr: &Expr, _ident: Option<String>) -> Option<String> {
        match expr {
            Expr::Call {
                name,
                args,
                return_type: _,
            } => {
                let arg_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                let mut temps: Vec<String> = Vec::new();
                for (idx, a) in args.iter().enumerate() {
                    let r = self
                        .handle_expr(a, None)
                        .expect("argument should yield a register");
                    temps.push(r);
                    if idx >= arg_regs.len() {
                        panic!("More than 6 arguments not supported yet");
                    }
                }
                for (i, t) in temps.iter().enumerate() {
                    self.output
                        .push_str(&format!("mov {} , {}\n", arg_regs[i], t));
                }
                // Free temps used to compute arguments; callee owns arg regs
                for t in temps {
                    self.regs.push_back(t);
                }
                // Record as extern candidate
                self.externs.insert(name.clone());
                let is_defined = self.functions.iter().any(|(n, _, _)| n == name);
                let target = if is_defined {
                    name.clone()
                } else {
                    format!("_{name}")
                };
                self.output.push_str(&format!("call {target}\n"));
                // Return in rax for non-void; hand back rax as value
                Some("rax".to_string())
            }

            Expr::BoolLiteral(n) => {
                let val = match n {
                    true => 1,
                    false => 0,
                };
                let av_reg = self.regs.pop_front().expect("No registers available");
                self.output.push_str(&format!("mov {av_reg}, {val}\n"));
                Some(av_reg.to_string())
            }
            Expr::IntLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers available");
                self.output.push_str(&format!("mov {av_reg}, {n}\n"));
                Some(av_reg.to_string())
            }
            Expr::Variable(name) => {
                let off = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown variable '{name}'"));
                let av_reg = self.regs.pop_front().expect("No registers available");
                self.output
                    .push_str(&format!("mov {av_reg}, QWORD [rbp - {off}]\n"));
                Some(av_reg)
            }
            Expr::ArrayAccess { array, index } => {
                let (_name, base_off) = match &**array {
                    Expr::Variable(n) => {
                        let off = *self
                            .locals
                            .get(n)
                            .unwrap_or_else(|| panic!("Could not find array {n}?!"));
                        (n, off)
                    }
                    _ => panic!("ArrayAccess: only local stack arrays are supported right now"),
                };

                // Result register
                let dst = self.regs.pop_front().expect("No registers available");

                match &**index {
                    Expr::IntLiteral(n) => {
                        let off = base_off - (*n as i32) * 8;
                        self.output
                            .push_str(&format!("mov {dst}, QWORD [rbp - {off}]\n"));
                        Some(dst)
                    }

                    _ => {
                        let idx = self
                            .handle_expr(index, None)
                            .expect("index should produce a register");
                        // mov dst, [rbp + idx*8 - base_off]
                        self.output
                            .push_str(&format!("mov {dst}, QWORD [rbp + {idx}*8 - {base_off}]\n"));
                        // free idx temp
                        self.regs.push_back(idx);
                        Some(dst)
                    }
                }
            }

            // Expr::Array(elements, _ty) => {
            //     for element in elements {
            //         self.handle_expr(element, None);
            //     }

            //     None
            // }
            Expr::Assign { name, value } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown variable '{name}'"));

                match **value {
                    Expr::IntLiteral(n) => {
                        self.output
                            .push_str(&format!("mov QWORD [rbp - {offset}], {n}\n"));
                    }
                    Expr::BoolLiteral(b) => {
                        let val = if b { 1 } else { 0 };
                        self.output
                            .push_str(&format!("mov QWORD [rbp - {offset}], {val}\n"));
                    }
                    _ => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov QWORD [rbp - {offset}], {val_reg}\n"));
                            self.regs.push_back(val_reg);
                        }
                    }
                }

                None
            }
            // Expr::Unary {
            //     op,
            //     expr,
            //     result_type: _,
            // } => match op {
            //     UnaryOp::AddressOf => {
            //         Only support taking address of a variable for now
            //         if let Expr::Variable(var_name) = &**expr {
            //             let off = self
            //                 .local_offset(var_name)
            //                 .unwrap_or_else(|| panic!("Unknown variable '{var_name}'"));
            //             let av_reg = self.regs.pop_front().expect("No registers available");
            //             self.output
            //                 .push_str(&format!("lea {av_reg}, [rbp - {off}]\n"));
            //             Some(av_reg)
            //         } else {
            //             panic!("Address-of is only supported on variables for now");
            //         }
            //     }
            //     UnaryOp::Dereference => {
            //         Load value from pointer
            //         let ptr_reg = self
            //             .handle_expr(expr, None)
            //             .expect("Pointer expression should yield a register");
            //         self.output
            //             .push_str(&format!("mov {ptr_reg}, QWORD [{ptr_reg}]\n"));
            //         Some(ptr_reg)
            //     }
            //     UnaryOp::Not => {
            //         let reg = self.handle_expr(expr, None).unwrap();
            //         self.output.push_str(&format!("cmp {reg}, 0\n"));
            //         self.output.push_str("sete al\n");
            //         self.output.push_str(&format!("movzx {reg}, al\n"));
            //         Some(reg)
            //     }
            //     UnaryOp::Negate => {
            //         let reg = self.handle_expr(expr, None).unwrap();
            //         self.output.push_str(&format!("neg {reg}\n"));
            //         Some(reg)
            //     }
            // },
            // in handle_expr(...)
            Expr::Unary {
                op: UnaryOp::AddressOf,
                expr,
                ..
            } => match &**expr {
                Expr::Unary {
                    op: UnaryOp::Dereference,
                    expr: inner,
                    ..
                } => self.handle_expr(inner, None),

                Expr::Variable(var_name) => {
                    let off = self
                        .local_offset(var_name)
                        .unwrap_or_else(|| panic!("Unknown variable '{var_name}'"));
                    let reg = self.regs.pop_front().expect("No registers available");
                    self.output.push_str(&format!("lea {reg}, [rbp - {off}]\n"));
                    Some(reg)
                }

                Expr::ArrayAccess { array, index, .. } => {
                    let base_off = *self
                        .locals
                        .get(match &**array {
                            Expr::Variable(n) => n,
                            _ => panic!("ArrayAccess base must be a local array for now"),
                        })
                        .expect("unknown array");
                    let idx_reg = match &**index {
                        Expr::IntLiteral(n) => {
                            let r = self.regs.pop_front().expect("No registers");
                            self.output.push_str(&format!("mov {r}, {n}\n"));
                            r
                        }
                        _ => self.handle_expr(index, None).expect("index reg"),
                    };
                    let addr = self.regs.pop_front().expect("No registers");
                    self.output
                        .push_str(&format!("lea {addr}, [rbp + {idx_reg}*8 - {base_off}]\n"));
                    self.regs.push_back(idx_reg);
                    Some(addr)
                }

                _ => panic!("Address-of only supported on lvalues for now"),
            },

            Expr::Unary {
                op: UnaryOp::Dereference,
                expr,
                ..
            } => {
                match &**expr {
                    // *&E  ==>  E
                    Expr::Unary {
                        op: UnaryOp::AddressOf,
                        expr: inner,
                        ..
                    } => self.handle_expr(inner, None),
                    _ => {
                        // generic *ptr load
                        let ptr = self.handle_expr(expr, None).expect("pointer reg");
                        self.output.push_str(&format!("mov {ptr}, QWORD [{ptr}]\n"));
                        Some(ptr)
                    }
                }
            }

            Expr::DerefAssign { target, value } => {
                let ptr_reg = self
                    .handle_expr(target, None)
                    .expect("Pointer target should yield a register");
                if let Some(val_reg) = self.handle_expr(value, None) {
                    self.output
                        .push_str(&format!("mov QWORD [{ptr_reg}], {val_reg}\n"));
                    // free temps
                    self.regs.push_back(ptr_reg);
                    self.regs.push_back(val_reg);
                } else {
                    // free ptr reg
                    self.regs.push_back(ptr_reg);
                }
                None
            }

            Expr::Binary {
                left,
                op,
                right,
                result_type: _,
            } => {
                let lhs = self.handle_expr(left, None).unwrap();
                let rhs = self.handle_expr(right, None).unwrap();
                match op {
                    BinaryOp::Add => {
                        self.output.push_str(&format!("add {lhs}, {rhs}\n"));
                        // free rhs
                        self.regs.push_back(rhs);
                        return Some(lhs);
                    }
                    BinaryOp::Sub => {
                        self.output.push_str(&format!("sub {lhs}, {rhs}\n"));
                        self.regs.push_back(rhs);
                        return Some(lhs);
                    }
                    BinaryOp::Mul => {
                        self.output.push_str(&format!("imul {lhs}, {rhs}\n"));
                        self.regs.push_back(rhs);
                        return Some(lhs);
                    }
                    BinaryOp::Div => {
                        // Use rax/rdx for division
                        self.output.push_str(&format!("mov rax, {lhs}\n"));
                        self.output.push_str("xor rdx, rdx\n");
                        self.output.push_str(&format!("div {rhs}\n"));
                        // result in rax
                        // free temps lhs, rhs
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        // keep rax as result
                        return Some("rax".to_string());
                    }
                    _ => {}
                }
                None
            }
            _ => None,
        }
    }
}
