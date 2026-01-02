use crate::{
    analyzer::base_type,
    backend::x86_64::{
        CodeGen,
        regs::{reg8, reg32},
    },
    lexer::ast::{Expr, Stmt, Type, UnaryOp},
};

impl CodeGen {
    pub fn handle_stmt_with_epilogue(&mut self, stmt: &Stmt, epilogue: &str) {
        match stmt {
            Stmt::Return(opt) => {
                if epilogue == ".Lret_main" {
                    self.output.push_str("mov rdi, 10\n");
                    self.call_with_alignment("print_char");

                    self.output.push_str("mov rdi, rbx\n");
                }
                if let Some(ex) = opt {
                    if let Some(reg) = self.handle_expr(ex, None) {
                        if reg != "rax" {
                            self.output.push_str("xor rax, rax\n");

                            self.output.push_str(&format!("mov rax, {reg}\n"));
                            self.regs.push_back(reg);
                        }
                    } else {
                        self.output.push_str("xor rax, rax\n");
                    }
                }

                self.output.push_str(&format!("jmp {epilogue}\n"));
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_reg = self.handle_expr(condition, None).expect("if cond reg");
                let jmp_id = self._jmp_count;
                self._jmp_count += 1;

                self.output
                    .push_str(&format!("cmp {cond_reg}, 0\nje .else{jmp_id}\n"));
                self.regs.push_back(cond_reg);

                self.handle_stmt_with_epilogue(then_stmt, epilogue);
                if else_stmt.is_some() {
                    self.output.push_str(&format!("jmp .endif{jmp_id}\n"));
                }

                self.output.push_str(&format!(".else{jmp_id}:\n"));
                if let Some(else_s) = else_stmt {
                    self.handle_stmt_with_epilogue(else_s, epilogue);
                    self.output.push_str(&format!(".endif{jmp_id}:\n"));
                }
            }
            Stmt::While { condition, body } => {
                let jmp_id = self._jmp_count;
                self._jmp_count += 1;

                let loop_start = format!(".while_start_{}", jmp_id);
                let loop_end = format!(".while_end_{}", jmp_id);

                self.output.push_str(&format!("{loop_start}:\n"));

                let condition_reg = self
                    .handle_expr(condition, None)
                    .expect("Error handling condition");
                self.output.push_str(&format!("cmp {condition_reg}, 1\n"));
                self.output.push_str(&format!("jne {loop_end}\n"));
                self.regs.push_back(condition_reg);

                self.handle_stmt_with_epilogue(body, epilogue);

                self.output.push_str(&format!("jmp {loop_start}\n"));
                self.output.push_str(&format!("{loop_end}:\n"));
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.handle_stmt_with_epilogue(s, epilogue);
                }
            }
            _ => self.handle_stmt(stmt),
        }
    }

    pub fn handle_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::AtDecl(decl, params, _, _) => {
                if decl.as_str() == "__asm__" || decl.as_str() == "_asm_" || decl.as_str() == "asm"
                {
                    self.output
                        .push_str(&params.clone().unwrap_or("".to_string()));
                }
                if decl.as_str() == "__asm_ro__"
                    || decl.as_str() == "_asm_ro_"
                    || decl.as_str() == "asm_ro"
                {
                    self.rodata.push_str(&params.clone().unwrap());
                }
                if decl.as_str() == "__asm_bss__"
                    || decl.as_str() == "_asm_bss_"
                    || decl.as_str() == "asm_bss"
                {
                    self.bss.push_str(&params.clone().unwrap());
                }
            }
            Stmt::While { condition, body } => {
                let jmp_id = self._jmp_count;
                self._jmp_count += 1;

                let loop_start = format!(".while_start_{}", jmp_id);
                let loop_end = format!(".while_end_{}", jmp_id);

                self.output.push_str(&format!("{loop_start}:\n"));

                // evaluate condition
                let condition_reg = self
                    .handle_expr(condition, None)
                    .expect("Error handling condition");
                self.output.push_str(&format!("cmp {condition_reg}, 1\n"));
                self.output.push_str(&format!("jne {loop_end}\n"));
                self.regs.push_back(condition_reg);

                // eval body
                self.handle_stmt(body);

                // loop back

                // self.output
                //     .push_str(&format!("add rsp, {}\n", self.stack_size));

                self.output.push_str(&format!("jmp {loop_start}\n"));

                // exit
                self.output.push_str(&format!("{loop_end}:\n"));
            }

            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => match value {
                Expr::Cast { expr, target_type } => {
                    let reg = self.handle_expr(expr, None).unwrap();
                    let off = self.alloc_local(name, target_type);

                    match target_type {
                        Type::int => self
                            .output
                            .push_str(&format!("mov dword [rbp - {off}], {}\n", reg32(&reg))),

                        Type::Bool | Type::Char => self
                            .output
                            .push_str(&format!("mov byte [rbp - {off}], {}\n", reg8(&reg))),

                        Type::float => self
                            .output
                            .push_str(&format!("movss [rbp - {off}], {reg}\n")),

                        _ => self
                            .output
                            .push_str(&format!("mov qword [rbp - {off}], {reg}\n")),
                    }

                    self.regs.push_back(reg);
                }
                Expr::Array(elements, ty) => {
                    let mut count = elements.len();

                    if let Type::Array(_, len) = var_type {
                        count = len.unwrap();
                    }

                    let total_size = (count as i32) * 8;
                    self.stack_size += total_size;
                    self.output.push_str(&format!("sub rsp, {total_size}\n"));
                    let base_offset = self.stack_size;
                    self.locals.insert(
                        name.clone(),
                        (
                            None,
                            base_offset,
                            Type::Array(Box::new(ty.clone()), Some(elements.len())),
                        ),
                    );

                    // > TODO : Offset array members by type size, not always 8 <

                    for (i, element) in elements.iter().enumerate() {
                        let offset = base_offset - (i as i32) * 8;
                        match element {
                            Expr::FloatLiteral(f) => {
                                self.output
                                    .push_str(&format!("movss [rbp - {offset}], {f}\n"));
                            }
                            Expr::IntLiteral(n) => {
                                self.output
                                    .push_str(&format!("mov dword [rbp - {offset}], {n}\n"));
                            }
                            Expr::BoolLiteral(b) => {
                                let val = if *b { 1 } else { 0 };
                                self.output
                                    .push_str(&format!("mov byte [rbp - {offset}], {val}\n"));
                            }
                            Expr::CharLiteral(c) => {
                                self.output
                                    .push_str(&format!("mov byte [rbp - {offset}], '{c}'\n"));
                            }
                            Expr::StringLiteral(str) => {
                                let reg = self
                                    .handle_expr(&Expr::StringLiteral(str.to_string()), None)
                                    .expect("Unable to handle string?");

                                self.output
                                    .push_str(&format!("mov qword [rbp - {offset}], {reg}\n"));

                                self.regs.push_back(reg);
                            }
                            _ => {
                                if let Some(val_reg) = self.handle_expr(element, None) {
                                    self.output.push_str(&format!(
                                        "mov qword [rbp - {offset}], {val_reg}\n"
                                    ));

                                    self.regs.push_back(val_reg);
                                }
                            }
                        }
                    }

                    if let Type::Array(_, len) = var_type {
                        let left = len.unwrap() - elements.len();
                        if left > 0 {
                            for i in elements.len()..elements.len() + left {
                                let offset = base_offset - (i as i32) * 8;

                                self.output
                                    .push_str(&format!("mov dword [rbp - {offset}], 0\n"));
                            }
                        }
                    }
                }

                Expr::StringLiteral(str) => {
                    let offset = self.alloc_local(name, &Type::Pointer(Box::new(Type::Char)));

                    let reg = self
                        .handle_expr(&Expr::StringLiteral(str.to_string()), None)
                        .expect("Unable to handle string?");

                    self.output
                        .push_str(&format!("mov qword [rbp - {offset}], {reg}\n"));

                    self.regs.push_back(reg);
                }

                Expr::StructInit { name: cls, params } => {
                    // Get the struct layout to map named parameters to field positions
                    // let struct_info = self.structures.get(cls);
                    // if struct_info.is_none() {
                    //     panic!("Struct {} not found", cls);
                    // }
                    // let (struct_layout, _, _) = struct_info.unwrap();

                    // // Clone the field names and positions to avoid borrowing issues
                    // let field_positions: HashMap<String, usize> = struct_layout
                    //     .fields
                    //     .iter()
                    //     .enumerate()
                    //     .map(|(i, field)| (field.name.clone(), i))
                    //     .collect();
                    // let field_count = struct_layout.fields.len();

                    // // First, evaluate all expressions to get their register values
                    // let mut param_values = Vec::new();
                    // for (param_name, param_expr) in params {
                    //     let r = self.handle_expr(param_expr, None).expect("ctor arg");
                    //     param_values.push((param_name, r));
                    // }

                    // // Map named parameters to their correct positions
                    // let mut ordered_args = vec![String::new(); field_count];
                    // for (param_name, reg) in param_values {
                    //     if let Some(&pos) = field_positions.get(param_name) {
                    //         ordered_args[pos] = reg;
                    //     } else {
                    //         panic!("Field {} not found in struct {}", param_name, cls);
                    //     }
                    // }

                    // // Filter out empty strings and get the actual argument values
                    // let arg_vals: Vec<String> =
                    //     ordered_args.into_iter().filter(|s| !s.is_empty()).collect();

                    // let abi_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];
                    // if arg_vals.len() > abi_regs.len() {
                    //     panic!("More than 5 constructor args not supported yet");
                    // }

                    // for (i, r) in arg_vals.iter().enumerate() {
                    //     if r != abi_regs[i] {
                    //         self.output
                    //             .push_str(&format!("mov {}, {}\n", abi_regs[i], r));
                    //     }
                    // }
                    // for r in arg_vals {
                    //     self.regs.push_back(r);
                    // }

                    // // let ctor = format!("{cls}.new");
                    // // self.call_with_alignment(&ctor);

                    // self.output.push_str(&self.structures.get(cls).unwrap_or_else(|| panic!("Struct {cls} is not in struct list")).2);

                    // let struc = Type::Struct {
                    //     name: cls.to_string(),
                    //     instances: params
                    //         .iter()
                    //         .map(|(name, expr)| (name.clone(), expr.get_type()))
                    //         .collect(),
                    // };

                    let reg = self
                        .handle_expr(
                            &Expr::StructInit {
                                name: cls.to_string(),
                                params: params.to_vec(),
                            },
                            None,
                        )
                        .unwrap();

                    let off = self.alloc_local(
                        name,
                        &Type::Struct {
                            name: cls.to_string(),
                            instances: params
                                .iter()
                                .map(|(i, j)| (i.clone(), j.get_type()))
                                .collect(),
                        },
                    );

                    // println!("allocated {struct:?}");

                    self.output
                        .push_str(&format!("mov qword [rbp - {off}], {reg}\n"));

                    self.regs.push_back(reg);
                }
                Expr::AddressOf(inside) => {
                    if let Type::Struct { .. } = base_type(var_type) {
                        self.handle_expr(inside, None);

                        let offset = if self.local_offset(name).is_none() {
                            self.alloc_local(name, var_type)
                        } else {
                            self.local_offset(name).unwrap()
                        };

                        self.output.push_str(&format!(
                            "mov qword [rbp - {offset}], [rbp - {offset} + 8]\n"
                        ));
                    } else {
                        let offset = if self.local_offset(name).is_none() {
                            self.alloc_local(name, var_type)
                        } else {
                            self.local_offset(name).unwrap()
                        };

                        self.output.push_str(&format!(
                            "mov qword [rbp - {offset}], [rbp - {offset} + 8]\n"
                        ));
                    }
                }
                Expr::Unary {
                    op: UnaryOp::Dereference,
                    expr,
                    ..
                } => {
                    let ptr_reg = self.handle_expr(expr, None).expect("ptr reg");
                    let offset = self.alloc_local(name, var_type);

                    if let Type::Pointer(inner) = var_type
                        && let Type::Struct {
                            name: struct_name, ..
                        } = &**inner
                        {
                            self.locals.insert(
                                name.clone(),
                                (Some(struct_name.clone()), offset, var_type.clone()),
                            );
                        }

                    self.output
                        .push_str(&format!("mov {ptr_reg}, [{ptr_reg}]\n"));

                    self.output
                        .push_str(&format!("mov qword [rbp - {offset}], {ptr_reg}\n"));

                    self.regs.push_back(ptr_reg);
                }
                // Expr::InstanceVar(struct_name, field_name) => {
                //     let struct = self
                //         .locals
                //         .get(struct_name)
                //         .expect(&format!("error getting struct: {struct_name}"))
                //         .0
                //         .clone()
                //         .expect(&format!("error getting struct: {struct_name}"));

                //     let offset = if self.local_offset(struct_name).is_none() {
                //         self.alloc_local(name, None, var_type)
                //     } else {
                //         self.local_offset(name).unwrap()
                //     };

                //     println!("{:?}", self.locals);

                //     let field_offset: usize = self
                //         .structures
                //         .get(&struct)
                //         .expect("error getting struct layout")
                //         .0
                //         .fields
                //         .iter()
                //         .find_map(|field| {
                //             if field.name == *field_name {
                //                 Some(field.offset)
                //             } else {
                //                 None
                //             }
                //         })
                //         .unwrap();

                //     if let Some(val_reg) = self.handle_expr(value, None) {
                //         match var_type {
                //             Type::int => {
                //                 self.output.push_str(&format!(
                //                     "mov dword [rbp - {offset} - {field_offset}], {}\n",
                //                     reg32(&val_reg)
                //                 ));
                //             }
                //             Type::Bool => {
                //                 self.output.push_str(&format!(
                //                     "mov byte [rbp - {offset} - {field_offset}], {}\n",
                //                     reg8(&val_reg)
                //                 ));
                //             }
                //             Type::Char => {
                //                 self.output.push_str(&format!(
                //                     "mov dword [rbp - {offset} - {field_offset}], {}\n",
                //                     reg8(&val_reg)
                //                 ));
                //             }
                //             _ => {
                //                 self.output
                //                     .push_str(&format!("mov qword [rbp - {offset}], {val_reg}\n",));
                //             }
                //         }

                //         self.regs.push_back(val_reg);
                //     }
                // }
                Expr::InstanceVar(struct_name, field_name) => {
                    // Get the struct pointer from the local variable
                    let struct_info = self
                        .locals
                        .get(struct_name)
                        .unwrap_or_else(|| panic!("Struct '{}' not found in locals", struct_name));

                    let struct_type = struct_info
                        .0
                        .clone()
                        .unwrap_or_else(|| panic!("Struct type not found for '{}'", struct_name));

                    let struct_offset = struct_info.1;

                    // Get struct layout
                    let struct_layout = &self
                        .structures
                        .get(&struct_type)
                        .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                        .0;

                    // Find field offset
                    let mut field_offset = struct_layout
                        .fields
                        .iter()
                        .find(|f| f.name == *field_name)
                        .unwrap_or_else(|| {
                            panic!(
                                "Field '{}' not found in struct '{}'",
                                field_name, struct_type
                            )
                        })
                        .offset;

                    let struct_stmt = &self
                        .structures
                        .get(struct_type.trim())
                        .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                        .1;

                    if let Stmt::StructDecl { union, .. } = struct_stmt
                        && *union {
                            field_offset = 0;
                        }

                    // Allocate space for the variable if not already allocated
                    let offset = if self.local_offset(name).is_none() {
                        self.alloc_local(name, var_type)
                    } else {
                        self.local_offset(name).unwrap()
                    };

                    // Get the field value from the struct instance
                    let ptr_reg = self.regs.pop_front().expect("No registers available");
                    self.output
                        .push_str(&format!("mov {ptr_reg}, qword [rbp - {struct_offset}]\n"));

                    match var_type {
                        Type::int => {
                            self.output.push_str(&format!(
                                "mov eax, dword [{ptr_reg} + {field_offset}]\n"
                            ));
                            self.output
                                .push_str(&format!("mov dword [rbp - {offset}], eax\n"));
                        }
                        Type::Char | Type::Bool => {
                            self.output
                                .push_str(&format!("mov al, byte [{ptr_reg} + {field_offset}]\n"));
                            self.output
                                .push_str(&format!("mov byte [rbp - {offset}], al\n"));
                        }
                        // Type::Struct { name, instances } => {
                        //     // Clone the instances to avoid borrowing self.structures during the call
                        //     let struct_name = name.clone();
                        //     let struct_instances = instances.to_vec();
                        //     let stack_size = self.stack_size.try_into().unwrap();
                        //     self.generate_stack_struct_inline(
                        //         &struct_name,
                        //         struct_instances,
                        //         stack_size,
                        //         None,
                        //     );
                        // }
                        _ => {
                            self.output.push_str(&format!(
                                "mov rax, qword [{ptr_reg} + {field_offset}]\n"
                            ));
                            self.output
                                .push_str(&format!("mov qword [rbp - {offset}], rax\n"));
                        }
                    }

                    self.regs.push_back(ptr_reg);
                }
                _ => {
                    // let mut struct = None;

                    // if let Expr::Call { return_type, .. } = value {
                    //     match return_type {
                    //         Type::Struct { name, .. } => {
                    //             struct = Some(name);
                    //         }
                    //         _ => {}
                    //     }
                    // }

                    let offset = if self.local_offset(name).is_none() {
                        self.alloc_local(name, var_type)
                    } else {
                        self.local_offset(name).unwrap()
                    };

                    match var_type {
                        Type::float => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                self.output
                                    .push_str(&format!("movss [rbp - {offset}], {val_reg}\n"));
                                self.fp_regs.push_back(val_reg);
                            }
                        }
                        _ => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                match value.get_type() {
                                    Type::int => self.output.push_str(&format!(
                                        "mov dword [rbp - {offset}], {}\n",
                                        reg32(&val_reg)
                                    )),
                                    Type::Char | Type::Bool => self.output.push_str(&format!(
                                        "mov byte [rbp - {offset}], {}\n",
                                        reg8(&val_reg)
                                    )),
                                    _ => self.output.push_str(&format!(
                                        "mov qword [rbp - {offset}], {val_reg}\n"
                                    )),
                                }

                                self.regs.push_back(val_reg);
                            }
                        }
                    }
                }
            },

            // Stmt::StructDecl {
            //     name,
            //     instances,
            //     funcs,
            // } => {}
            Stmt::Expression(expr) => {
                if let Some(reg) = self.handle_expr(expr, None) {
                    self.regs.push_back(reg);
                }
            }

            Stmt::Return(Some(ex)) => {
                self.handle_expr(ex, None);
            }
            Stmt::Return(None) => {
                self.output.push_str("ret\n");
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                let cond_reg = self.handle_expr(condition, None).expect("if cond reg");
                let jmp_id = self._jmp_count;
                self._jmp_count += 1;

                self.output
                    .push_str(&format!("cmp {cond_reg}, 0\nje .else{jmp_id}\n"));
                self.regs.push_back(cond_reg);

                self.handle_stmt(then_stmt);
                if else_stmt.is_some() {
                    self.output.push_str(&format!("jmp .endif{jmp_id}\n"));
                }

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
}
