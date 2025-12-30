use std::collections::HashMap;

use crate::{
    codegen::{
        CodeGen, align_up,
        regs::{reg8, reg32},
        size_align_of,
    },
    lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
};

impl CodeGen {
    pub fn handle_expr(&mut self, expr: &Expr, _ident: Option<String>) -> Option<String> {
        match expr {
            Expr::Cast { expr, target_type } => {
                let src = self.handle_expr(expr, None)?;
                match target_type {
                    Type::int => {
                        if src.starts_with("xmm") {
                            let dst = self.regs.pop_front().expect("No gp reg");
                            self.output
                                .push_str(&format!("cvttss2si {}, {}\n", reg32(&dst), src));
                            self.fp_regs.push_back(src);
                            Some(dst)
                        } else {
                            Some(src)
                        }
                    }
                    Type::float => {
                        if src.starts_with("xmm") {
                            Some(src)
                        } else {
                            let dst = self.fp_regs.pop_front().expect("No fp reg");
                            self.output
                                .push_str(&format!("cvtsi2ss {dst}, {}\n", reg32(&src)));
                            self.regs.push_back(src);
                            Some(dst)
                        }
                    }
                    _ => Some(src),
                }
            }
            Expr::IndexAssign {
                array,
                index,
                value,
            } => {
                if let Expr::Variable(name, _) = *array.clone() {
                    let var = self.locals.get(&name).unwrap();
                    let off = var.1;
                    let ty = match var.2 {
                        Type::int => "dword",
                        Type::Char | Type::Bool => "byte",
                        _ => "qword",
                    };

                    let val_reg = self.handle_expr(value, None).unwrap();

                    let index_reg = self.handle_expr(index, None).unwrap();

                    let reg = self.regs.pop_front().unwrap();

                    self.output.push_str(&format!(
                        "imul {index_reg}, 8\nmov {reg}, rbp\nadd {reg}, {index_reg}\nsub {reg}, {off}\nmov {ty} [{reg}], {val_reg}\n"
                    ));

                    self.regs.push_back(reg);
                    self.regs.push_back(index_reg);
                    self.regs.push_back(val_reg);
                }

                None
            }
            Expr::LongLiteral(l) => {
                let reg = self.regs.pop_front().unwrap();

                self.output.push_str(&format!("mov {reg}, {l}"));

                Some(reg)
            }
            Expr::StringLiteral(str) => {
                let callee = ["rbx", "r12", "r13", "r14", "r15"];

                let chs = Self::parse_escapes(str);

                self.output
                    .push_str(&format!("mov rdi, {}\n", chs.len() + 1));
                self.call_with_alignment("malloc");

                for (i, c) in chs.iter().enumerate() {
                    match c {
                        '\n' => {
                            self.output.push_str(&format!("mov byte [rax + {i}], 10\n"));
                        }
                        '\0' => {
                            self.output.push_str(&format!("mov byte [rax + {i}], 0\n"));
                        }
                        _ => {
                            self.output
                                .push_str(&format!("mov byte [rax + {i}], '{c}'\n"));
                        }
                    }
                }

                self.output
                    .push_str(&format!("mov byte [rax + {}], 0\n", chs.len()));

                let mut reg = None;

                if let Some(pos) = self.regs.iter().position(|r| callee.contains(&r.as_str())) {
                    reg = Some(self.regs[pos].clone());
                    self.regs.remove(pos);
                }

                self.output
                    .push_str(&format!("mov {}, rax\n", reg.clone().unwrap()));

                reg
            }
            // alloc to stack -> pointer to loc
            Expr::FieldAssign {
                class_name,
                field,
                value,
            } => {
                // Get struct information from locals
                let struct_info = self
                    .locals
                    .get(class_name)
                    .unwrap_or_else(|| panic!("Struct '{}' not found", class_name));

                // Get struct name and stack offset
                let struct_type = struct_info
                    .0
                    .clone()
                    .unwrap_or_else(|| panic!("Struct type not found for '{}'", class_name));

                let struct_offset = struct_info.1;

                // Get struct layout from structures registry
                let struct_layout = &self
                    .structures
                    .get(&struct_type)
                    .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                    .0;

                // Find field in struct layout
                let mut field_offset = struct_layout
                    .fields
                    .iter()
                    .find(|f| f.name == *field)
                    .map(|f| f.offset)
                    .unwrap_or_else(|| {
                        panic!("Field '{}' not found in struct '{}'", field, struct_type)
                    });

                // Get field type from struct declaration
                let field_type = {
                    let struct_decl_stmt = &self.structures.get(&struct_type).unwrap().1;
                    if let Stmt::StructDecl {
                        instances, union, ..
                    } = struct_decl_stmt
                    {
                        if *union {
                            field_offset = 0;
                        }

                        instances
                            .iter()
                            .find(|(fname, _)| fname == field)
                            .map(|(_, ftype)| ftype.clone())
                            .unwrap_or_else(|| panic!("Field type not found for '{}'", field))
                    } else {
                        panic!("Struct declaration not found for '{}'", struct_type)
                    }
                };

                // Handle the value expression
                let val_reg = self
                    .handle_expr(value, None)
                    .expect("Could not find register for field value");

                // println!("{value:?}");

                // Calculate field address based on struct location

                let reg = self.regs.pop_front().unwrap();

                self.output
                    .push_str(&format!("mov {reg}, [rbp - {struct_offset}]\n"));

                // Generate appropriate store instruction based on field type
                match field_type {
                    Type::int => {
                        self.output.push_str(&format!(
                            "mov dword [{reg} + {field_offset}], {}\n",
                            reg32(&val_reg)
                        ));
                    }
                    Type::Bool | Type::Char => {
                        self.output.push_str(&format!(
                            "mov byte [{reg} + {field_offset}], {}\n",
                            reg8(&val_reg)
                        ));
                    }
                    Type::float => {
                        self.output
                            .push_str(&format!("movss [{reg} + {field_offset}], {val_reg}\n"));
                        // return fp reg to pool
                        self.fp_regs.push_back(val_reg.to_string());
                        // also keep reg for address below
                        self.regs.push_back(reg.clone());
                        return None;
                    }
                    _ => {
                        self.output.push_str(&format!(
                            "mov qword [{reg} + {field_offset}], {}\n",
                            val_reg
                        ));
                    }
                }

                // Return value register to pool
                self.regs.push_back(val_reg.to_string());
                self.regs.push_back(reg);

                None
            }
            Expr::Array(elements, ty) => {
                let size_1 = size_align_of(ty);
                let size = size_1.0 * elements.len();
                let size = align_up(size, size_1.1);

                self.output.push_str(&format!(
                    "sub rsp, 8\npush rdi\nmov rdi, {size}\nadd rsp, 8\n"
                ));

                #[cfg(target_arch = "aarch64")]
                self.call_with_alignment("_malloc");

                #[cfg(target_arch = "x86_64")]
                self.call_with_alignment("malloc");

                // let _ = self.regs.clone().iter().enumerate().map(|(i, _)| {
                //     if self.regs[i] == "rax" {
                //         self.regs.remove(i);
                //     }
                // });

                let reg = self
                    .regs
                    .pop_front()
                    .unwrap_or_else(|| panic!("No register"));

                self.output.push_str(&format!("mov {reg}, rax\n"));
                //self.output.push_str("mov r15, rax\n");

                Some(reg)
            }
            Expr::Call { name, args, .. } => {
                let abi_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];

                if name == "sizeof" {
                    if args.len() > 1 {
                        panic!("sizeof takes 1 arg");
                    }

                    let mut size = args[0].get_type().size();

                    if let Expr::Variable(struct_name, _) = &args[0] {
                        // println!("{:?}", self.structures.get(struct_name).unwrap().0.fields);
                        size = self.structures.get(struct_name).unwrap().0._size;
                    }

                    // println!("{args:?}");


                    for (i, reg) in self.regs.clone().iter().enumerate() {
                        if reg == "rax" {
                            self.regs.remove(i);
                        }
                    }

                    self.output.push_str(&format!("mov rax, {size}\n"));

                    let reg = self.regs.pop_front().unwrap();
                    self.output.push_str(&format!("mov {reg}, rax\n"));
                    return Some(reg);
                }

                let mut temps: Vec<String> = Vec::new();
                for (idx, a) in args.iter().enumerate() {
                    let r = self.handle_expr(a, None).expect("arg reg");
                    temps.push(r);
                    if idx >= abi_regs.len() {
                        panic!("More than 5 arguments not supported yet");
                    }
                }

                for (i, t) in temps.iter().enumerate() {
                    if t != abi_regs[i] {
                        self.output
                            .push_str(&format!("mov {}, {}\n", abi_regs[i], t));
                    }
                }

                for t in temps {
                    self.regs.push_back(t);
                }
                // self.externs.insert(name.clone());
                let is_defined = self.functions.iter().any(|(n, _, _, _)| n == name);
                let target = if is_defined {
                    name.clone()
                } else {
                    format!("{name}")
                };

                self.call_with_alignment(&target);

                let reg = self.regs.pop_front().unwrap();
                self.output.push_str(&format!("mov {reg}, rax\n"));
                return Some(reg);
            }
            Expr::FloatLiteral(f) => {
                self.output.push_str(&format!(
                    "section .data\nfp{}: dd {f}\nsection .text\n",
                    self.fp_count
                ));

                let reg = self.fp_regs.pop_front().expect("No fp reg");

                self.output
                    .push_str(&format!("movss {reg}, [fp{}]\n", self.fp_count));

                self.fp_count += 1;

                Some(reg)
            }
            Expr::CharLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, '{n}'\n", reg8(&av_reg)));
                if *n == '\0' {
                    self.output.push_str(&format!("mov {}, 0\n", av_reg));
                } else {
                    self.output.push_str(&format!("mov {}, '{n}'\n", av_reg));
                }

                Some(av_reg)
            }

            Expr::BoolLiteral(b) => {
                let val = if *b { 1 } else { 0 };
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, {val}\n", reg8(&av_reg)));
                self.output.push_str(&format!("mov {}, {val}\n", av_reg));
                Some(av_reg)
            }

            Expr::IntLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, {n}\n", reg32(&av_reg)));
                self.output.push_str(&format!("mov {}, {n}\n", av_reg));

                Some(av_reg)
            }
            Expr::InstanceVar(struct_name, instance_name) => {
                // let av_reg = self.regs.pop_front().expect("No registers");

                // // Get the struct pointer from the local variable
                // let struct_ptr_off = self
                //     .local_offset(struct_name)
                //     .expect("Error reading struct name");

                // // Load the pointer to the struct instance
                // self.output
                //     .push_str(&format!("mov {av_reg}, qword [rbp - {struct_ptr_off}]\n"));

                // // Get struct type information
                // let struct_type = self
                //     .locals
                //     .get(struct_name)
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     })
                //     .0
                //     .clone()
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     });

                // let struct_layout = &self
                //     .structures
                //     .get(struct_type.trim())
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     })
                //     .0;

                // // Find the field offset
                // let mut field_offset = None;
                // for field in &struct_layout.fields {
                //     if &field.name == instance_name {
                //         field_offset = Some(field.offset);
                //         break;
                //     }
                // }

                // let av_reg = self.regs.pop_front().expect("No registers");
                // let mut off = self
                //     .local_offset(struct_name)
                //     .expect("Error reading struct name");

                // let struct_ptr_reg = self
                //     .handle_expr(&Expr::Variable(struct_name.to_string()), None)
                //     .unwrap_or_else(|| panic!("Could not locate: '{struct_name}'"));

                // let struct_type = self
                //     .locals
                //     .get(struct_name)
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     })
                //     .0
                //     .clone()
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     });

                // let struct_layout = &self
                //     .structures
                //     .get(struct_type.trim())
                //     .unwrap_or_else(|| {
                //         panic!("Error parsing struct type for variable: {struct_name}")
                //     })
                //     .0;

                // for field in &struct_layout.fields {
                //     if &field.name == instance_name {
                //         off += field.offset as i32;
                //     }
                // }

                // self.output
                //     .push_str(&format!("mov {av_reg}, qword [{av_reg} - {off}]\n"));

                // self.regs.push_front(struct_ptr_reg);

                // Some(av_reg)

                // let obj_ptr_off = self
                //     .local_offset(struct_name)
                //     .expect("Error reading struct name");
                // let obj_ptr_reg = self.regs.pop_front().expect("No registers");

                // // load the object pointer from local variable
                // self.output
                //     .push_str(&format!("mov {obj_ptr_reg}, qword [rbp - {obj_ptr_off}]\n"));

                // // compute field offset
                // let mut field_offset = 0;
                // for field in &struct_layout.fields {
                //     if &field.name == instance_name {
                //         field_offset = field.offset;
                //         break;
                //     }
                // }

                // // load field value
                // self.output.push_str(&format!(
                //     "mov {obj_ptr_reg}, qword [{obj_ptr_reg} + {field_offset}]\n"
                // ));

                // Some(obj_ptr_reg)

                // let cls = self.locals.get(struct_name).unwrap().0.clone().unwrap();
                // let stmt = self.structures.get(&cls).unwrap().1.clone();
                // if let Stmt::StructDecl {
                //     name,
                //     instances,
                //     union,
                // } = stmt
                // {
                //     if union {
                //         let off = self.locals.get(struct_name).unwrap().1;
                //         for (name1, ty) in instances {
                //             if name1 == name {
                //                 match ty {
                //                     _ => {}
                //                 }
                //             }
                //         }
                //     }
                // }

                let val_reg = self.regs.pop_front().expect("No registers available");

                // println!("{:?}", self.locals);

                let struct_type = self
                    .locals
                    .get(struct_name)
                    .unwrap_or_else(|| {
                        panic!("Error getting struct type for variable '{struct_name}'")
                    })
                    .0
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("Struct type not found for variable '{struct_name}'")
                    });

                let ptr_reg = self
                    .handle_expr(
                        &Expr::Variable(struct_name.to_string(), Type::Unknown),
                        None,
                    )
                    .expect("Could not load struct pointer");

                let struct_layout = &self
                    .structures
                    .get(struct_type.trim())
                    .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                    .0;

                let mut field_offset = None;
                let mut field_type = None;
                for field in &struct_layout.fields {
                    if &field.name == instance_name {
                        field_offset = Some(field.offset as i32);
                        // Get the field type from the struct definition
                        if let Some((_, struct_stmt)) = self.structures.get(struct_type.trim()) {
                            if let Stmt::StructDecl { instances, .. } = struct_stmt {
                                for (fname, ftype) in instances {
                                    if fname == instance_name {
                                        field_type = Some(ftype);
                                        break;
                                    }
                                }
                            }
                        }
                        break;
                    }
                }

                let mut field_offset = field_offset.expect(&format!(
                    "Field '{}' not found in struct '{}'",
                    instance_name, struct_type
                ));

                let field_type = field_type.expect(&format!(
                    "Field type for '{}' not found in struct '{}'",
                    instance_name, struct_type
                ));

                let struct_stmt = &self
                    .structures
                    .get(struct_type.trim())
                    .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                    .1;

                if let Stmt::StructDecl { union, .. } = struct_stmt {
                    if *union {
                        field_offset = 0;
                    }
                }

                match field_type {
                    Type::int => {
                        self.output.push_str(&format!(
                            "mov {}, dword [{ptr_reg} + {field_offset}]\n",
                            reg32(&val_reg)
                        ));
                    }
                    Type::Char | Type::Bool => {
                        self.output.push_str(&format!(
                            "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                            reg8(&val_reg)
                        ));
                    }
                    Type::float => {
                        let xmm = self.fp_regs.pop_front().expect("No fp reg");
                        self.output
                            .push_str(&format!("movss {xmm}, [{ptr_reg} + {field_offset}]\n"));
                        // return the gp val_reg we borrowed
                        self.regs.push_back(val_reg);
                        // and keep ptr_reg live below
                        self.regs.push_back(ptr_reg.clone());
                        return Some(xmm);
                    }
                    _ => {
                        self.output.push_str(&format!(
                            "mov {val_reg}, qword [{ptr_reg} + {field_offset}]\n"
                        ));
                    }
                }
                let _field_type = {
                    let struct_decl_stmt = &self.structures.get(&struct_type).unwrap().1;
                    if let Stmt::StructDecl { instances, .. } = struct_decl_stmt {
                        instances
                            .iter()
                            .find(|(fname, _)| fname == instance_name)
                            .map(|(_, ftype)| ftype.clone())
                            .unwrap_or_else(|| {
                                panic!("Field type not found for '{}'", instance_name)
                            })
                    } else {
                        panic!("Struct declaration not found for '{}'", struct_type)
                    }
                };

                self.regs.push_back(ptr_reg);
                Some(val_reg)
            }
            Expr::Variable(name, _) => {
                // println!("{:?}", self.regs);
                // Decide register class based on type
                if let Some(t) = self.globals.get(name) {
                    match t {
                        Type::float => {
                            let xmm = self.fp_regs.pop_front().expect("No fp reg");
                            self.output.push_str(&format!("movss {xmm}, [{name}]\n"));
                            return Some(xmm);
                        }
                        _ => {}
                    }
                }

                let av_reg = self.regs.pop_front().expect("No registers");

                self.output.push_str(&format!("xor {av_reg}, {av_reg}\n"));

                if let Some(t) = self.globals.get(name) {
                    match t {
                        Type::int => {
                            self.output
                                .push_str(&format!("mov {}, [{name}]\n", reg32(&av_reg)));
                            return Some(av_reg);
                        }
                        Type::Char | Type::Bool => {
                            self.output
                                .push_str(&format!("mov {}, [{name}]\n", reg8(&av_reg)));
                            return Some(av_reg);
                        }
                        Type::Pointer(_) | Type::Long | Type::Struct { .. } => {
                            self.output.push_str(&format!("mov {av_reg}, {name}\n"));
                            return Some(av_reg);
                        }
                        Type::float => {
                            // already handled above
                            unreachable!()
                        }
                        _ => {}
                    }
                }

                let off = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                let t = &self.locals.get(name).unwrap().2;

                match t {
                    Type::int => {
                        self.output
                            .push_str(&format!("mov {}, dword [rbp - {off}]\n", reg32(&av_reg)));
                        Some(av_reg)
                    }
                    Type::float => {
                        // Use XMM register
                        let xmm = self.fp_regs.pop_front().expect("No fp reg");
                        self.output
                            .push_str(&format!("movss {xmm}, [rbp - {off}]\n"));
                        // Return FP reg instead of GP
                        self.regs.push_back(av_reg);
                        Some(xmm)
                    }
                    Type::Char | Type::Bool => {
                        self.output
                            .push_str(&format!("mov {}, byte [rbp - {off}]\n", reg8(&av_reg)));
                        Some(av_reg)
                    }
                    _ => {
                        self.output
                            .push_str(&format!("mov {av_reg}, qword [rbp - {off}]\n"));
                        Some(av_reg)
                    }
                }
            }
            Expr::ArrayAccess { array, index, .. } => {
                let (_name, base_off) = match &**array {
                    Expr::Variable(n, _) => {
                        let off = self
                            .locals
                            .get(n)
                            .unwrap_or_else(|| panic!("No array {n}"))
                            .1;
                        (n, off)
                    }
                    _ => panic!("ArrayAccess only supports local arrays"),
                };
                let dst = self.regs.pop_front().expect("No registers");

                match &**index {
                    Expr::IntLiteral(n) => {
                        let off = base_off - n * 8;
                        self.output
                            .push_str(&format!("mov {dst}, qword [rbp - {off}]\n"));
                        Some(dst)
                    }
                    _ => {
                        let idx = self.handle_expr(index, None).expect("index reg");
                        self.output
                            .push_str(&format!("mov {dst}, qword [rbp - {idx}*8 - {base_off}]\n"));
                        self.regs.push_back(idx);
                        Some(dst)
                    }
                }
            }

            Expr::StructInit { name, params } => {
                let (field_names, field_types_ordered): (Vec<String>, Vec<Type>) = {
                    let (layout, stmt) = self
                        .structures
                        .get(name)
                        .unwrap_or_else(|| panic!("Struct {} not found", name));

                    let names: Vec<String> = layout.fields.iter().map(|f| f.name.clone()).collect();

                    let mut map: HashMap<&str, &Type> = HashMap::new();
                    if let Stmt::StructDecl { instances, .. } = stmt {
                        for (fname, fty) in instances {
                            map.insert(fname.as_str(), &fty);
                        }
                    }

                    #[allow(suspicious_double_ref_op)]
                    let tys: Vec<Type> = names
                        .iter()
                        .map(|n| map.get(n.as_str()).unwrap().clone().clone())
                        .collect();

                    (names, tys)
                };

                let field_count = field_names.len();
                let field_positions: HashMap<String, usize> = field_names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), i))
                    .collect();

                let mut param_values = Vec::new();
                for (param_name, param_expr) in params {
                    let r = self.handle_expr(param_expr, None).expect("ctor arg");
                    match param_expr.get_type() {
                        Type::float => {
                            if !r.starts_with("xmm") {
                                let xmm = self.fp_regs.pop_front().expect("No fp reg");
                                self.output.push_str(&format!("cvtsi2ss {xmm}, {}\n", r));
                                self.regs.push_back(r);
                                param_values.push((param_name, xmm));
                                continue;
                            }
                        }
                        Type::int | Type::Char | Type::Bool | Type::Pointer(_) | Type::Long => {
                            if r.starts_with("xmm") {
                                let gp = self.regs.pop_front().expect("No gp reg");
                                self.output
                                    .push_str(&format!("cvttss2si {}, {}\n", reg32(&gp), r));
                                self.fp_regs.push_back(r);
                                param_values.push((param_name, gp));
                                continue;
                            }
                        }
                        _ => {}
                    }
                    param_values.push((param_name, r));
                }

                let mut ordered_args = vec![String::new(); field_count];
                for (param_name, reg) in param_values {
                    if let Some(&pos) = field_positions.get(param_name) {
                        ordered_args[pos] = reg;
                    } else {
                        panic!("Field {} not found in struct {}", param_name, name);
                    }
                }

                let arg_vals: Vec<String> =
                    ordered_args.into_iter().filter(|s| !s.is_empty()).collect();

                let gp_targets = ["rdi", "rsi", "rdx", "rcx", "r8"];
                let fp_targets = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4"];
                let mut gp_i = 0usize;
                let mut fp_i = 0usize;

                for (idx, _) in field_names.iter().enumerate() {
                    let src = &arg_vals[idx];
                    match field_types_ordered[idx] {
                        Type::float => {
                            let dst = fp_targets[fp_i];
                            if src != dst {
                                // move between XMM registers
                                self.output.push_str(&format!("movss {dst}, {}\n", src));
                            }
                            fp_i += 1;
                        }
                        Type::int
                        | Type::Char
                        | Type::Bool
                        | Type::Pointer(_)
                        | Type::Long
                        | Type::Struct { .. } => {
                            let dst = gp_targets[gp_i];
                            if src != dst {
                                self.output.push_str(&format!("mov {}, {}\n", dst, src));
                            }
                            gp_i += 1;
                        }
                        _ => {
                            let dst = gp_targets[gp_i];
                            if src != dst {
                                self.output.push_str(&format!("mov {}, {}\n", dst, src));
                            }
                            gp_i += 1;
                        }
                    }
                }

                for r in arg_vals {
                    self.regs.push_back(r);
                }

                let off = self.stack_size.max(8);

                // self.output.push_str(&self.structures.get(name).unwrap().2);

                let op = &self.generate_stack_struct_inline(
                    name,
                    params
                        .iter()
                        .map(|(name, expr)| (name.clone(), expr.get_type()))
                        .collect(),
                    off.try_into().unwrap(),
                    None,
                );

                self.output.push_str(op);

                // let ctor = format!("{name}.new");

                // self.call_with_alignment(&ctor);

                let _struct = 
                //Type::Pointer(Box::new(
                    Type::Struct {
                    name: name.to_string(),
                    instances: params
                        .iter()
                        .map(|(name, expr)| (name.clone(), expr.get_type()))
                        .collect(),
                };
                //));

                // println!("allocated {struct:?}");

                // let reg = self.regs.pop_front().expect("No regs");

                // self.output.push_str(&format!("mov {reg}, rax\n"));

                let reg = self.regs.pop_front().unwrap();

                self.output.push_str(&format!("lea {reg}, [rsp]\n"));

                Some(reg)
            }

            Expr::Assign { name, value } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));
                match *value.clone() {
                    Expr::Variable(x, _) => {
                        let var_off = self
                            .local_offset(&x)
                            .expect(&format!("Error with variable: {x}"));

                        // match ty {
                        //     Type::int | Type::float => {}
                        //     Type::Char | Type::Bool => {}
                        //     _ =>
                        // }

                        let reg = self.regs.pop_front().unwrap();

                        self.output
                            .push_str(&format!("mov {reg}, [rbp - {var_off}]\n"));

                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], {reg}\n"));

                        self.regs.push_back(reg);
                    }
                    Expr::FloatLiteral(f) => {
                        self.output.push_str(&format!(
                            "section .data\nfp{}: dd {f}\nsection .text\n",
                            self.fp_count
                        ));

                        let reg = self.fp_regs.pop_front().expect("No fp regs");

                        self.output
                            .push_str(&format!("movss {reg}, [fp{}]", self.fp_count));

                        self.fp_count += 1;

                        self.output
                            .push_str(&format!("movss [rbp - {offset}], {reg} "));

                        self.fp_regs.push_back(reg);
                    }
                    Expr::IntLiteral(n) => {
                        self.output
                            .push_str(&format!("mov dword [rbp - {offset}], {n}\n"));
                    }
                    Expr::BoolLiteral(b) => {
                        let val = if b { 1 } else { 0 };
                        self.output
                            .push_str(&format!("mov byte [rbp - {offset}], {val}\n"));
                    }
                    Expr::CharLiteral(c) => {
                        self.output
                            .push_str(&format!("mov byte [rbp - {offset}], {c}\n"));
                    }
                    Expr::StringLiteral(str) => {
                        let reg = self
                            .handle_expr(&Expr::StringLiteral(str.to_string()), None)
                            .expect("Unable to handle string?");

                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], {reg}\n"));

                        self.regs.push_back(reg);
                    }
                    // Expr::InstanceVar(struct_name, instance_name) => {
                    //     let struct_type = self
                    //         .locals
                    //         .get(&struct_name)
                    //         .unwrap_or_else(|| panic!("Struct '{}' not found", struct_name))
                    //         .0
                    //         .clone()
                    //         .unwrap_or_else(|| panic!("Struct type not found for '{}'", struct_name));

                    //     let struct_layout = &self
                    //         .structures
                    //         .get(struct_type.trim())
                    //         .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                    //         .0;

                    //     let mut field_offset = None;
                    //     let mut field_type = None;
                    //     for field in &struct_layout.fields {
                    //         if &field.name == &instance_name {
                    //             field_offset = Some(field.offset as i32);
                    //             // Get the field type from the struct definition
                    //             if let Some((_, struct_stmt)) = self.structures.get(struct_type.trim()) {
                    //                 if let Stmt::StructDecl { instances, .. } = struct_stmt {
                    //                     for (fname, ftype) in instances {
                    //                         if fname == &instance_name {
                    //                             field_type = Some(ftype);
                    //                             break;
                    //                         }
                    //                     }
                    //                 }
                    //             }
                    //             break;
                    //         }
                    //     }

                    //     let _field_offset = field_offset.expect(&format!(
                    //         "Field '{}' not found in struct '{}'",
                    //         instance_name, struct_type
                    //     ));

                    //     let field_type = field_type.expect(&format!(
                    //         "Field type for '{}' not found in struct '{}'",
                    //         instance_name, struct_type
                    //     ));

                    //     match field_type {
                    //         Type::int => {
                    //             if let Some(val_reg) = self.handle_expr(value, None) {
                    //                 self.output.push_str(&format!(
                    //                     "mov dword [rbp - {offset}], {val_reg}\n"
                    //                 ));
                    //                 self.regs.push_back(val_reg);
                    //             }
                    //         }
                    //         Type::Char => {
                    //             if let Some(val_reg) = self.handle_expr(value, None) {
                    //                 self.output.push_str(&format!(
                    //                     "mov byte [rbp - {offset}], {val_reg}\n"
                    //                 ));
                    //                 self.regs.push_back(val_reg);
                    //             }
                    //         }
                    //         Type::Bool => {
                    //             if let Some(val_reg) = self.handle_expr(value, None) {
                    //                 self.output.push_str(&format!(
                    //                     "mov byte [rbp - {offset}], {val_reg}\n"
                    //                 ));
                    //                 self.regs.push_back(val_reg);
                    //             }
                    //         }
                    //         _ => {
                    //             if let Some(val_reg) = self.handle_expr(value, None) {
                    //                 self.output.push_str(&format!(
                    //                     "mov qword [rbp - {offset}], {val_reg}\n"
                    //                 ));
                    //                 self.regs.push_back(val_reg);
                    //             }
                    //         }
                    //     }
                    // }
                    Expr::InstanceVar(struct_name, field_name) => {
                        // Get struct pointer from local variable
                        let struct_ptr_off = self
                            .local_offset(&struct_name)
                            .expect("Struct variable not found");
                        let struct_ptr_reg = self.regs.pop_front().expect("No register available");
                        self.output.push_str(&format!(
                            "mov {struct_ptr_reg}, qword [rbp - {struct_ptr_off}]\n"
                        ));

                        // Get struct layout and field information

                        // self.locals.get(&struct_name).unwrap().;

                        let struct_type = self.locals.get(&struct_name).unwrap().0.clone().unwrap();
                        let struct_layout = &self.structures.get(&struct_type).unwrap().0;

                        let mut field_offset = struct_layout
                            .fields
                            .iter()
                            .find(|f| f.name == *field_name)
                            .unwrap()
                            .offset;

                        let val_reg = self.handle_expr(value, None).expect("No value register");

                        let struct_stmt = &self
                            .structures
                            .get(struct_type.trim())
                            .unwrap_or_else(|| {
                                panic!("Struct layout not found for '{struct_type}'")
                            })
                            .1;

                        if let Stmt::StructDecl { union, .. } = struct_stmt {
                            if *union {
                                field_offset = 0;
                            }
                        }

                        match value.get_type() {
                            Type::int => {
                                self.output.push_str(&format!(
                                    "mov dword [{struct_ptr_reg} + {field_offset}], {}\n",
                                    reg32(&val_reg)
                                ));
                            }
                            Type::Char | Type::Bool => {
                                self.output.push_str(&format!(
                                    "mov byte [{struct_ptr_reg} + {field_offset}], {}\n",
                                    reg8(&val_reg)
                                ));
                            }
                            Type::Struct { name, instances } => {
                                self.generate_stack_struct_inline(
                                    &name,
                                    instances,
                                    self.stack_size.try_into().unwrap(),
                                    None,
                                );
                            }
                            _ => {
                                self.output.push_str(&format!(
                                    "mov qword [{struct_ptr_reg} + {field_offset}], {val_reg}\n"
                                ));
                            }
                        }

                        self.regs.push_back(struct_ptr_reg);
                        self.regs.push_back(val_reg);
                    }
                    _ => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov qword [rbp - {offset}], {val_reg}\n"));
                            self.regs.push_back(val_reg);
                        }
                    }
                }
                None
            }

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
                Expr::Variable(var_name, _) => {
                    // println!("ref to {var_name}");
                    let off = self
                        .local_offset(var_name)
                        .unwrap_or_else(|| panic!("Unknown var '{var_name}'"));
                    let reg = self.regs.pop_front().expect("No registers");
                    self.output.push_str(&format!("lea {reg}, [rbp - {off}]\n"));
                    self.stack_size += 8;
                    Some(reg)
                }
                Expr::ArrayAccess { array, index, .. } => {
                    let base_off = self
                        .locals
                        .get(match &**array {
                            Expr::Variable(n, _) => n,
                            _ => panic!("Array base must be var"),
                        })
                        .expect("Error accessing array")
                        .1;
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
                        .push_str(&format!("lea {addr}, [rbp - {idx_reg}*8 - {base_off}]\n"));
                    self.regs.push_back(idx_reg);
                    Some(addr)
                }
                Expr::StructInit { name, params } => {
                    // Get the struct layout to map named parameters to field positions
                    let struct_info = self.structures.get(name);

                    if struct_info.is_none() {
                        panic!("Struct {} not found", name);
                    }

                    let (struct_layout, _) = struct_info.unwrap();

                    // Clone the field names and positions to avoid borrowing issues
                    let field_positions: HashMap<String, usize> = struct_layout
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| (field.name.clone(), i))
                        .collect();
                    let field_count = struct_layout.fields.len();

                    // First, evaluate all expressions to get their register values
                    let mut param_values = Vec::new();
                    for (param_name, param_expr) in params {
                        let r = self.handle_expr(param_expr, None).expect("ctor arg");
                        param_values.push((param_name, r));
                    }

                    // Map named parameters to their correct positions
                    let mut ordered_args = vec![String::new(); field_count];
                    for (param_name, reg) in param_values {
                        if let Some(&pos) = field_positions.get(param_name) {
                            ordered_args[pos] = reg;
                        } else {
                            panic!("Field {} not found in struct {}", param_name, name);
                        }
                    }

                    // Filter out empty strings and get the actual argument values
                    let arg_vals: Vec<String> =
                        ordered_args.into_iter().filter(|s| !s.is_empty()).collect();

                    let abi_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];
                    if arg_vals.len() > abi_regs.len() {
                        panic!("More than 5 constructor args not supported yet");
                    }

                    for (i, r) in arg_vals.iter().enumerate() {
                        if r != abi_regs[i] {
                            self.output
                                .push_str(&format!("mov {}, {}\n", abi_regs[i], r));
                        }
                    }
                    for r in arg_vals {
                        self.regs.push_back(r);
                    }

                    let ctor = format!("{name}.new");
                    self.call_with_alignment(&ctor);

                    let _struct = &Type::Pointer(Box::new(Type::Struct {
                        name: name.to_string(),
                        instances: params
                            .iter()
                            .map(|(name, expr)| (name.clone(), expr.get_type()))
                            .collect(),
                    }));

                    // println!("allocated {struct:?}");

                    let reg = self.regs.pop_front().unwrap();

                    self.output.push_str(&format!("lea {reg}, [rsp]\n"));

                    Some(reg)
                }
                Expr::InstanceVar(struct_name, field_name) => {
                    let obj_ptr_reg = self
                        .handle_expr(&Expr::Variable(struct_name.clone(), Type::Unknown), None)
                        .expect("Could not load struct pointer");

                    let struct_type = self
                        .locals
                        .get(struct_name)
                        .unwrap_or_else(|| panic!("Struct '{}' not found", struct_name))
                        .0
                        .clone()
                        .unwrap_or_else(|| panic!("Struct type not found for '{}'", struct_name));

                    let struct_layout = &self
                        .structures
                        .get(&struct_type)
                        .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                        .0;

                    let mut field_offset = struct_layout
                        .fields
                        .iter()
                        .find(|f| &f.name == field_name)
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

                    if let Stmt::StructDecl { union, .. } = struct_stmt {
                        if *union {
                            field_offset = 0;
                        }
                    }

                    // Get a register for the address
                    let addr_reg = self.regs.pop_front().expect("No registers available");

                    // Calculate address: obj_ptr + field_offset
                    self.output.push_str(&format!(
                        "lea {addr_reg}, [{obj_ptr_reg} + {field_offset}]\n"
                    ));

                    // Return the object pointer register to the pool
                    self.regs.push_back(obj_ptr_reg);

                    Some(addr_reg)
                }
                _ => panic!("Address-of only on lvalues"),
            },

            Expr::Unary {
                op: UnaryOp::Dereference,
                expr,
                ..
            } => match &**expr {
                Expr::Unary {
                    op: UnaryOp::AddressOf,
                    expr: inner,
                    ..
                } => self.handle_expr(inner, None),
                Expr::Unary {
                    op: UnaryOp::Dereference,
                    expr: inner,
                    ..
                } => self.handle_expr(inner, None),
                Expr::Variable(_, ty) => {
                    let ptr = self.handle_expr(expr, None).expect("ptr reg");

                    // match ty {
                    //     Type::int | Type::float => todo!(),

                    //     Type::Char | Type::Bool => todo!(),
                    //     _ => {}
                    // }

                    let reg = self.regs.pop_front().unwrap();

                    match ty {
                        Type::Bool | Type::Char => {
                            self.output
                                .push_str(&format!("mov {}, byte [{ptr}]\n", reg8(&reg)));
                        }
                        Type::int => {
                            self.output
                                .push_str(&format!("mov {}, dword [{ptr}]\n", reg32(&reg)));
                        }
                        Type::Pointer(inside) => match **inside {
                            Type::Bool | Type::Char => {
                                self.output
                                    .push_str(&format!("mov {}, byte [{ptr}]\n", reg8(&reg)));
                            }
                            Type::int => {
                                self.output
                                    .push_str(&format!("mov {}, dword [{ptr}]\n", reg32(&reg)));
                            }

                            _ => {
                                self.output.push_str(&format!("mov {reg}, qword [{ptr}]\n"));
                            }
                        },
                        _ => {
                            self.output.push_str(&format!("mov {reg}, qword [{ptr}]\n"));
                        }
                    }

                    self.regs.push_back(ptr);

                    Some(reg)
                }
                Expr::InstanceVar(struct_name, instance_name) => {
                    let val_reg = self.regs.pop_front().expect("No registers available");

                    let struct_type = self
                        .locals
                        .get(struct_name)
                        .unwrap_or_else(|| {
                            panic!("Error getting struct type for variable '{struct_name}'")
                        })
                        .0
                        .clone()
                        .unwrap_or_else(|| {
                            panic!("Struct type not found for variable '{struct_name}'")
                        });

                    let ptr_reg = self
                        .handle_expr(
                            &Expr::Variable(struct_name.to_string(), Type::Unknown),
                            None,
                        )
                        .expect("Could not load struct pointer");

                    let struct_layout = &self
                        .structures
                        .get(struct_type.trim())
                        .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                        .0;

                    let mut field_offset = None;
                    let mut field_type = None;
                    for field in &struct_layout.fields {
                        if &field.name == instance_name {
                            field_offset = Some(field.offset as i32);
                            // Get the field type from the struct definition
                            if let Some((_, struct_stmt)) = self.structures.get(struct_type.trim())
                            {
                                if let Stmt::StructDecl { instances, .. } = struct_stmt {
                                    for (fname, ftype) in instances {
                                        if fname == instance_name {
                                            field_type = Some(ftype);
                                            break;
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }

                    let mut field_offset = field_offset.expect(&format!(
                        "Field '{}' not found in struct '{}'",
                        instance_name, struct_type
                    ));

                    let field_type = field_type.expect(&format!(
                        "Field type for '{}' not found in struct '{}'",
                        instance_name, struct_type
                    ));

                    let struct_stmt = &self
                        .structures
                        .get(struct_type.trim())
                        .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                        .1;

                    if let Stmt::StructDecl { union, .. } = struct_stmt {
                        if *union {
                            field_offset = 0;
                        }
                    }

                    match field_type {
                        Type::int => {
                            self.output.push_str(&format!(
                                "mov {}, dword [{ptr_reg} + {field_offset}]\n",
                                reg32(&val_reg)
                            ));
                        }
                        Type::Char | Type::Bool => {
                            self.output.push_str(&format!(
                                "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                                reg8(&val_reg)
                            ));
                        }
                        Type::float => {
                            let xmm = self.fp_regs.pop_front().expect("No fp reg");
                            self.output
                                .push_str(&format!("movss {xmm}, [{ptr_reg} + {field_offset}]\n"));
                            self.regs.push_back(val_reg);
                            return Some(xmm);
                        }
                        // Type::Struct { name, instances } => {
                        //     // Clone the data needed before calling the mutating method
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
                                "mov {val_reg}, qword [{ptr_reg} + {field_offset}]\n"
                            ));
                        }
                    }

                    self.output
                        .push_str(&format!("mov {ptr_reg}, qword [{val_reg}]\n"));

                    self.regs.push_back(val_reg);

                    Some(ptr_reg)
                }
                _ => self.handle_expr(expr, None),
            },

            Expr::DerefAssign { target, value } => {
                let ptr_reg = self.handle_expr(target, None).expect("ptr reg");

                match value.get_type() {
                    Type::int => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov dword [{ptr_reg}], {}\n", reg32(&val_reg)));
                            self.regs.push_back(ptr_reg);
                            self.regs.push_back(val_reg);
                        } else {
                            self.regs.push_back(ptr_reg);
                        }
                    }
                    Type::float => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("movss [{ptr_reg}], {val_reg}\n"));
                            self.regs.push_back(ptr_reg);
                            self.fp_regs.push_back(val_reg);
                        } else {
                            self.regs.push_back(ptr_reg);
                        }
                    }
                    Type::Bool | Type::Char => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov byte [{ptr_reg}], {}\n", reg8(&val_reg)));
                            self.regs.push_back(ptr_reg);
                            self.regs.push_back(val_reg);
                        } else {
                            self.regs.push_back(ptr_reg);
                        }
                    }
                    Type::Pointer(inside) => match *inside {
                        Type::int => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                self.output.push_str(&format!(
                                    "mov dword [{ptr_reg}], {}\n",
                                    reg32(&val_reg)
                                ));
                                self.regs.push_back(ptr_reg);
                                self.regs.push_back(val_reg);
                            } else {
                                self.regs.push_back(ptr_reg);
                            }
                        }
                        Type::float => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                self.output
                                    .push_str(&format!("movss [{ptr_reg}], {val_reg}\n"));
                                self.regs.push_back(ptr_reg);
                                self.fp_regs.push_back(val_reg);
                            } else {
                                self.regs.push_back(ptr_reg);
                            }
                        }
                        Type::Bool | Type::Char => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                self.output.push_str(&format!(
                                    "mov byte [{ptr_reg}], {}\n",
                                    reg8(&val_reg)
                                ));
                                self.regs.push_back(ptr_reg);
                                self.regs.push_back(val_reg);
                            } else {
                                self.regs.push_back(ptr_reg);
                            }
                        }
                        _ => {
                            if let Some(val_reg) = self.handle_expr(value, None) {
                                self.output
                                    .push_str(&format!("mov qword [{ptr_reg}], {val_reg}\n"));
                                self.regs.push_back(ptr_reg);
                                self.regs.push_back(val_reg);
                            } else {
                                self.regs.push_back(ptr_reg);
                            }
                        }
                    },
                    _ => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output
                                .push_str(&format!("mov qword [{ptr_reg}], {val_reg}\n"));
                            self.regs.push_back(ptr_reg);
                            self.regs.push_back(val_reg);
                        } else {
                            self.regs.push_back(ptr_reg);
                        }
                    }
                }

                None
            }

            Expr::CompoundAssign { name, op, value } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                // Load current value into a register
                let current_reg = self.regs.pop_front().expect("No register available");
                self.output.push_str(&format!(
                    "mov {}, dword [rbp - {offset}]\n",
                    reg32(&current_reg)
                ));

                // Generate code for the right-hand side value
                let value_reg = self
                    .handle_expr(value, None)
                    .expect("No register for value");

                // Perform the compound operation
                match op {
                    BinaryOp::Add => {
                        self.output.push_str(&format!(
                            "add {}, {}\n",
                            reg32(&current_reg),
                            reg32(&value_reg)
                        ));
                    }
                    BinaryOp::Sub => {
                        self.output.push_str(&format!(
                            "sub {}, {}\n",
                            reg32(&current_reg),
                            reg32(&value_reg)
                        ));
                    }
                    BinaryOp::Mul => {
                        self.output.push_str(&format!(
                            "imul {}, {}\n",
                            reg32(&current_reg),
                            reg32(&value_reg)
                        ));
                    }
                    BinaryOp::Div => {
                        // For division, we need to use eax and edx
                        self.output
                            .push_str(&format!("mov eax, {}\n", reg32(&current_reg)));
                        self.output.push_str("cdq\n"); // Sign extend eax to edx:eax
                        self.output
                            .push_str(&format!("idiv {}\n", reg32(&value_reg)));
                        self.output
                            .push_str(&format!("mov {}, eax\n", reg32(&current_reg)));
                    }
                    _ => panic!("Unsupported compound assignment operator: {:?}", op),
                }

                // Store the result back to the variable
                self.output.push_str(&format!(
                    "mov dword [rbp - {offset}], {}\n",
                    reg32(&current_reg)
                ));

                // Return registers to the pool
                self.regs.push_back(current_reg);
                self.regs.push_back(value_reg);

                None
            }

            Expr::PreIncrement { name } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                // Increment the variable in place and return the new value
                self.output
                    .push_str(&format!("inc dword [rbp - {offset}]\n"));

                let reg = self.regs.pop_front().expect("No register available");
                self.output
                    .push_str(&format!("mov {}, dword [rbp - {offset}]\n", reg32(&reg)));

                Some(reg)
            }

            Expr::PostIncrement { name } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                // Load current value, then increment
                let reg = self.regs.pop_front().expect("No register available");
                self.output
                    .push_str(&format!("mov {}, dword [rbp - {offset}]\n", reg32(&reg)));
                self.output
                    .push_str(&format!("inc dword [rbp - {offset}]\n"));

                Some(reg)
            }

            Expr::PreDecrement { name } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                // Decrement the variable in place and return the new value
                self.output
                    .push_str(&format!("dec dword [rbp - {offset}]\n"));

                let reg = self.regs.pop_front().expect("No register available");
                self.output
                    .push_str(&format!("mov {}, dword [rbp - {offset}]\n", reg32(&reg)));

                Some(reg)
            }

            Expr::PostDecrement { name } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));

                // Load current value, then decrement
                let reg = self.regs.pop_front().expect("No register available");
                self.output
                    .push_str(&format!("mov {}, dword [rbp - {offset}]\n", reg32(&reg)));
                self.output
                    .push_str(&format!("dec dword [rbp - {offset}]\n"));

                Some(reg)
            }

            Expr::Binary {
                left, op, right, ..
            } => {
                match left.get_type() {
                    Type::Pointer(inside_ty) => match right.get_type() {
                        Type::int => {
                            let size = inside_ty.size();

                            if right.get_type() == Type::int {
                                let lhs = self.handle_expr(left, None).unwrap();

                                if let Expr::IntLiteral(n) = &**right {
                                    let r = self.regs.pop_front().expect("No registers");

                                    self.output.push_str(&format!("mov {}, {}\n", reg32(&r), n));

                                    let rhs_reg = r;

                                    if size > 1 {
                                        self.output
                                            .push_str(&format!("imul {}, {}\n", rhs_reg, size));
                                    }

                                    match op {
                                        BinaryOp::Add => {
                                            self.output
                                                .push_str(&format!("add {}, {}\n", lhs, rhs_reg));
                                            self.regs.push_back(rhs_reg);
                                            return Some(lhs);
                                        }
                                        BinaryOp::Sub => {
                                            self.output
                                                .push_str(&format!("sub {}, {}\n", lhs, rhs_reg));
                                            self.regs.push_back(rhs_reg);
                                            return Some(lhs);
                                        }
                                        _ => return None,
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                match right.get_type() {
                    Type::Pointer(inside_ty) => match left.get_type() {
                        Type::int => {
                            let size = inside_ty.size();

                            if left.get_type() == Type::int {
                                let rhs = self.handle_expr(&right, None).unwrap();

                                if let Expr::IntLiteral(n) = &**left {
                                    let r = self.regs.pop_front().expect("No registers");
                                    self.output.push_str(&format!("mov {}, {}\n", r, n));
                                    let lhs_reg = r;

                                    if size > 1 {
                                        self.output
                                            .push_str(&format!("imul {}, {}\n", lhs_reg, size));
                                    }

                                    match op {
                                        BinaryOp::Add => {
                                            self.output
                                                .push_str(&format!("add {}, {}\n", rhs, lhs_reg));
                                            self.regs.push_back(lhs_reg);
                                            return Some(rhs);
                                        }
                                        BinaryOp::Sub => {
                                            self.output
                                                .push_str(&format!("sub {}, {}\n", rhs, lhs_reg));
                                            self.regs.push_back(lhs_reg);
                                            return Some(rhs);
                                        }
                                        _ => return None,
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                // Float operations path
                if left.get_type() == Type::float {
                    let lhs = self.handle_expr(left, None).unwrap();
                    let rhs = self.handle_expr(right, None).unwrap();

                    match op {
                        BinaryOp::Add => {
                            self.output.push_str(&format!("addss {lhs}, {rhs}\n"));
                            self.fp_regs.push_back(rhs);
                            return Some(lhs);
                        }
                        BinaryOp::Sub => {
                            self.output.push_str(&format!("subss {lhs}, {rhs}\n"));
                            self.fp_regs.push_back(rhs);
                            return Some(lhs);
                        }
                        BinaryOp::Mul => {
                            self.output.push_str(&format!("mulss {lhs}, {rhs}\n"));
                            self.fp_regs.push_back(rhs);
                            return Some(lhs);
                        }
                        BinaryOp::Div => {
                            self.output.push_str(&format!("divss {lhs}, {rhs}\n"));
                            self.fp_regs.push_back(rhs);
                            return Some(lhs);
                        }
                        BinaryOp::Equal => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("sete al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::NotEqual => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("setne al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::Less => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("setb al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::LessEqual => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("setbe al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::Greater => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("seta al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::GreaterEqual => {
                            self.output.push_str(&format!("ucomiss {lhs}, {rhs}\n"));
                            self.output.push_str("setae al\n");
                            self.output.push_str("movzx rax, al\n");
                            self.fp_regs.push_back(lhs);
                            self.fp_regs.push_back(rhs);
                            let reg = self.regs.pop_front().unwrap();
                            self.output.push_str(&format!("mov {reg}, rax\n"));
                            return Some(reg);
                        }
                        BinaryOp::And | BinaryOp::Or | BinaryOp::Mod => {}
                        // TODO: ADD BITWISE
                        _ => {}
                    }
                }

                // Evaluate LHS first, then spill to stack to protect across potential calls in RHS
                let lhs_saved = self.handle_expr(left, None).unwrap();
                self.output.push_str("sub rsp, 8\n");
                self.stack_size += 8;
                self.output
                    .push_str(&format!("mov qword [rsp], {}\n", lhs_saved));
                // release lhs reg back to pool while we compute RHS
                self.regs.push_back(lhs_saved.clone());

                let rhs = self.handle_expr(right, None).unwrap();

                // Reload LHS into a working register
                let lhs = self.regs.pop_front().unwrap();
                self.output.push_str(&format!("mov {}, qword [rsp]\n", lhs));
                self.output.push_str("add rsp, 8\n");
                self.stack_size -= 8;

                match op {
                    BinaryOp::Add => {
                        let lhs32 = reg32(&lhs);
                        let rhs32 = reg32(&rhs);
                        self.output.push_str(&format!("add {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Sub => {
                        let lhs32 = reg32(&lhs);
                        let rhs32 = reg32(&rhs);
                        self.output.push_str(&format!("sub {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Mul => {
                        let lhs32 = reg32(&lhs);
                        let rhs32 = reg32(&rhs);
                        self.output.push_str(&format!("imul {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Div => {
                        // Signed 32-bit division: edx:eax / r/m32 -> eax
                        let lhs32 = reg32(&lhs);
                        let rhs32 = reg32(&rhs);
                        self.output.push_str(&format!("mov eax, {lhs32}\n"));
                        self.output.push_str("cdq\n");
                        self.output.push_str(&format!("idiv {rhs32}\n"));
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::Equal => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("sete al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::NotEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setne al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::Less => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setl al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::LessEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setle al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::Greater => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setg al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::GreaterEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setge al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::And => {
                        let end_label = format!(".and_end_{}", self._jmp_count);
                        self._jmp_count += 1;

                        self.output.push_str(&format!("cmp {lhs}, 1\n"));
                        self.output.push_str(&format!("jne {end_label}\n"));
                        self.output.push_str(&format!("cmp {rhs}, 1\n"));
                        self.output.push_str(&format!("jne {end_label}\n"));
                        self.output.push_str("mov rax, 1\n");
                        self.output
                            .push_str(&format!("jmp .and_done_{}\n", self._jmp_count));
                        self.output.push_str(&format!("{end_label}:\n"));
                        self.output.push_str("mov rax, 0\n");
                        self.output
                            .push_str(&format!(".and_done_{}:\n", self._jmp_count));
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::Or => {
                        let end_label = format!(".or_end_{}", self._jmp_count);
                        self._jmp_count += 1;

                        self.output.push_str(&format!("cmp {lhs}, 1\n"));
                        self.output.push_str(&format!("je {end_label}\n"));
                        self.output.push_str(&format!("cmp {rhs}, 1\n"));
                        self.output.push_str(&format!("je {end_label}\n"));
                        self.output.push_str("mov rax, 0\n");
                        self.output
                            .push_str(&format!("jmp .or_done_{}\n", self._jmp_count));
                        self.output.push_str(&format!("{end_label}:\n"));
                        self.output.push_str("mov rax, 1\n");
                        self.output
                            .push_str(&format!(".or_done_{}:\n", self._jmp_count));
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    BinaryOp::Mod => {
                        self.output.push_str(&format!("mov rax, {lhs}\n"));
                        self.output.push_str("xor rdx, rdx\n");
                        self.output.push_str(&format!("div {rhs}\n"));
                        self.output.push_str("mov rax, rdx\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);

                        let reg = self.regs.pop_front().unwrap();
                        self.output.push_str(&format!("mov {reg}, rax\n"));
                        Some(reg)
                    }
                    // TODO: ADD BITWISE
                    _ => None,
                }
            }
            Expr::Unary {
                op: UnaryOp::Negate,
                expr,
                ..
            } => {
                match expr.get_type() {
                    Type::float => {
                        // Compute 0.0 - expr
                        let src = self.handle_expr(expr, None).expect("No reg for negate");
                        self.output.push_str(&format!(
                            "section .data\nfp{}: dd 0.0\nsection .text\n",
                            self.fp_count
                        ));
                        let tmp = self.fp_regs.pop_front().expect("No fp reg");
                        self.output
                            .push_str(&format!("movss {tmp}, [fp{}]\n", self.fp_count));
                        self.fp_count += 1;
                        self.output.push_str(&format!("subss {tmp}, {src}\n"));
                        self.fp_regs.push_back(src);
                        Some(tmp)
                    }
                    _ => {
                        let reg = self
                            .handle_expr(expr, None)
                            .expect("No reg for unary negate");
                        self.output.push_str(&format!("neg {reg}\n"));
                        Some(reg)
                    }
                }
            }
            _ => None,
        }
    }
}
