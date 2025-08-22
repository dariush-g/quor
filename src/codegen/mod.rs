use crate::{
    analyzer::base_type,
    lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
};

type Functions = Vec<(String, Vec<(String, Type)>, Vec<Stmt>)>;

pub struct CodeGen {
    output: String,
    // imports: Vec<String>,
    regs: VecDeque<String>,
    _fp_regs: VecDeque<String>,
    _jmp_count: u32,
    functions: Functions,
    locals: HashMap<String, (Option<String>, i32, Type)>,
    stack_size: i32,
    externs: HashSet<String>,
    structes: HashMap<String, (StructLayout, Stmt)>,
}

#[inline]
fn align_up(x: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two());
    (x + a - 1) & !(a - 1)
}

fn _gpr_name(i: usize, ty: &Type) -> &'static str {
    match ty {
        Type::int => match i {
            0 => "edi",
            1 => "esi",
            2 => "edx",
            3 => "ecx",
            4 => "r8d",
            _ => unreachable!(),
        },
        Type::Pointer(_) => match i {
            0 => "rdi",
            1 => "rsi",
            2 => "rdx",
            3 => "rcx",
            4 => "r8",
            _ => unreachable!(),
        },
        Type::Char | Type::Bool => match i {
            0 => "dil",
            1 => "sil",
            2 => "dl",
            3 => "cl",
            4 => "r8b",
            _ => unreachable!(),
        },
        _ => "rax",
    }
}

fn size_align_of(ty: &Type) -> (usize, usize) {
    match ty {
        Type::Char | Type::Bool => (1, 1),
        Type::int => (4, 4),
        Type::float => (4, 4),
        Type::Pointer(_) => (8, 8),
        // Type::Array(_elem, Some(_n)) => {
        // let (es, ea) = size_align_of(elem);
        // (es * *n, ea)
        //     (8, 8)
        // }
        Type::Array(_, _) => (8, 8),
        Type::Struct { instances, .. } => layout_of_struct(
            &instances
                .iter()
                .map(|(_, ty)| ty.clone())
                .collect::<Vec<Type>>(),
        ),
        Type::Void | Type::Unknown => (0, 1),
        _ => (0, 1),
    }
}

fn layout_of_struct(instances: &[Type]) -> (usize, usize) {
    if instances.is_empty() {
        return (0, 1);
    }
    let mut off = 0usize;
    let mut max_a = 1usize;
    for ty in instances {
        let (sz, al) = size_align_of(ty);
        let al = al.max(1);
        off = align_up(off, al);
        off += sz;
        max_a = max_a.max(al);
    }
    let size = align_up(off, max_a);
    (size, max_a)
}

struct FieldLayout {
    name: String,
    offset: usize,
}

struct StructLayout {
    size: usize,
    _align: usize,
    fields: Vec<FieldLayout>,
}

fn layout_fields(fields: &[(String, Type)]) -> StructLayout {
    let mut off = 0usize;
    let mut max_a = 1usize;
    let mut out = Vec::with_capacity(fields.len());

    for (name, ty) in fields {
        let (sz, al) = size_align_of(ty);
        let al = al.max(1);
        off = align_up(off, al);
        out.push(FieldLayout {
            name: name.clone(),
            offset: off,
        });
        off += sz;
        max_a = max_a.max(al);
    }

    let size = align_up(off, max_a);

    StructLayout {
        size,
        _align: max_a,
        fields: out,
    }
}

impl CodeGen {
    // asm      import paths
    pub fn generate(stmts: &Vec<Stmt>) -> String {
        let mut code = CodeGen {
            output: String::new(),
            regs: VecDeque::from(vec![
                "rcx".to_string(),
                "rdx".to_string(),
                "rax".to_string(),
                "r8".to_string(),
                "r9".to_string(),
                "r10".to_string(),
                "r11".to_string(),
                "r12".to_string(),
                "r13".to_string(),
                "r14".to_string(),
            ]),
            _fp_regs: VecDeque::from(vec![
                "xmm0".to_string(),
                "xmm1".to_string(),
                "xmm2".to_string(),
                "xmm3".to_string(),
                "xmm4".to_string(),
                "xmm5".to_string(),
                "xmm6".to_string(),
                "xmm7".to_string(),
            ]),
            _jmp_count: 0,
            functions: vec![],
            locals: HashMap::new(),
            stack_size: 0,
            externs: HashSet::new(),
            structes: HashMap::new(),
        };

        #[cfg(target_arch = "aarch64")]
        code.output.push_str("extern _malloc\n");
        #[cfg(target_arch = "x86_64")]
        code.output.push_str("extern malloc\n");

        code.output.push_str("global _start\n_start:\n");
        code.output.push_str("call main\n");
        code.output.push_str("mov rbx, rax\n");
        code.output.push_str("mov rdi, rax\n");

        #[cfg(target_arch = "aarch64")]
        code.output.push_str("mov rax, 0x2000001\n");
        #[cfg(target_arch = "x86_64")]
        code.output.push_str("mov rax, 60\n");
        code.output.push_str("syscall\n");

        let mut has_main = false;

        for stmt in stmts {
            if let Stmt::FunDecl {
                name, params, body, ..
            } = stmt
            {
                if name == "main" {
                    code.generate_function("main", vec![], body);
                    has_main = true;
                } else {
                    code.functions
                        .push((name.clone(), params.clone(), body.clone()));
                }
            }
            if let Stmt::StructDecl { name, instances } = stmt {
                code.generate_struct(name, instances.clone());
            }
        }

        if !has_main {
            panic!("No main function found");
        }

        let functions = &code.functions.clone();
        for (name, params, body) in functions {
            code.generate_function(name, params.clone(), body);
        }

        for stmt in stmts {
            if !matches!(stmt, Stmt::FunDecl { .. }) {
                code.handle_stmt(stmt);
            }
        }

        let defined: HashSet<String> = code.functions.iter().map(|(n, _, _)| n.clone()).collect();
        let mut header = String::new();

        #[cfg(target_arch = "aarch64")]
        for ext in code.externs.difference(&defined) {
            header.push_str(&format!("extern _{ext}\n"));
        }

        #[cfg(target_arch = "x86_64")]
        for ext in code.externs.difference(&defined) {
            header.push_str(&format!("extern {ext}\n"));
        }

        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let mut print = fs::read_to_string(format!("{manifest_dir}/src/stdlib/print.asm"))
            .unwrap_or_else(|_| panic!("Error importing io"));

        print.push('\n');

        code.output.push_str(&print);

        let mut mem = fs::read_to_string(format!("{manifest_dir}/src/stdlib/mem.asm"))
            .unwrap_or_else(|_| panic!("Error importing mem"));

        mem.push('\n');

        code.output.push_str(&mem);

        format!("{header}{}", code.output)
    }

    // fn alloc_local(&mut self, name: &str, struct_name: Option<String>, _ty: &Type) -> i32 {
    //     self.stack_size += 8;
    //     self.output.push_str("sub rsp, 8\n");
    //     let offset = self.stack_size;
    //     self.locals.insert(name.to_string(), (struct_name, offset));
    //     offset
    // }

    fn alloc_local(&mut self, name: &str, ty: &Type) -> i32 {
        self.stack_size += 8;
        self.output.push_str("sub rsp, 8\n");
        let offset = self.stack_size;

        let struct_info = match ty {
            Type::Pointer(inner) => {
                if let Type::Struct { name: cls_name, .. } = &**inner {
                    Some(cls_name.clone())
                } else {
                    None
                }
            }
            Type::Struct { name: cls_name, .. } => Some(cls_name.clone()),
            _ => None,
        };

        self.locals
            .insert(name.to_string(), (struct_info, offset, ty.clone()));
        offset
    }

    fn local_offset(&self, name: &str) -> Option<i32> {
        if self.locals.contains_key(name) {
            return Some(
                self.locals
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Error with local variable: {name}"))
                    .1,
            );
        }

        None
    }

    fn call_with_alignment(&mut self, target: &str) {
        let slots = self.stack_size / 8;
        let need_pad = (slots & 1) != 0;
        if need_pad {
            self.output.push_str("sub rsp, 8\n");
        }
        self.output.push_str(&format!("call {target}\n"));
        if need_pad {
            self.output.push_str("add rsp, 8\n");
        }
    }

    fn handle_stmt(&mut self, stmt: &Stmt) {
        match stmt {
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
                self.output.push_str(&format!("jmp {loop_start}\n"));

                // exit
                self.output.push_str(&format!("{loop_end}:\n"));
            }
            // Stmt::AtDecl(decl, param) => {
            //     // v v v v v v v v v v v v v v v v v v v
            //     //>//TODO: IMPROVE IMPORT HANDLING!!!! <
            //     // ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^

            //     if decl.as_str() == "import" {
            //         let mut param = param
            //             .clone()
            //             .unwrap_or_else(|| panic!("Unable to locate import"));

            //         if param.ends_with('!') {
            //             param.remove(param.len() - 1);

            //             let manifest_dir = env!("CARGO_MANIFEST_DIR");

            //             match param.as_str() {
            //                 _ => {}
            //             }
            //         } else {
            //             let file = fs::read_to_string(param.clone())
            //                 .unwrap_or_else(|_| panic!("Unable to find file {param}"));

            //             self.output.push_str(&file);
            //         }
            //     }
            // }
            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => match value {
                Expr::Array(elements, ty) => {
                    let count = elements.len();
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

                    for (i, element) in elements.iter().enumerate() {
                        let offset = base_offset - (i as i32) * 8;
                        match element {
                            Expr::IntLiteral(n) => {
                                self.output
                                    .push_str(&format!("mov qword [rbp - {offset}], {n}\n"));
                            }
                            Expr::BoolLiteral(b) => {
                                let val = if *b { 1 } else { 0 };
                                self.output
                                    .push_str(&format!("mov qword [rbp - {offset}], {val}\n"));
                            }
                            Expr::CharLiteral(c) => {
                                self.output
                                    .push_str(&format!("mov qword [rbp - {offset}], '{c}'\n"));
                            }
                            Expr::StringLiteral(str) => {
                                // let offset = self.alloc_local(
                                //     name,
                                //     &Type::Struct {
                                //         name: "string".to_owned(),
                                //         instances: vec![
                                //             ("size".to_string(), Type::int),
                                //             (
                                //                 "data".to_string(),
                                //                 Type::Pointer(Box::new(Type::Char)),
                                //             ),
                                //         ],
                                //     },
                                // );

                                self.output
                                    .push_str(&format!("mov rdi, {}\ncall malloc\n", str.len()));

                                // rdi - size
                                // rsi - char*

                                for (i, ch) in str.chars().enumerate() {
                                    let reg = self.regs.pop_front().expect("No regs");

                                    self.output
                                        .push_str(&format!("mov {}, '{ch}'\n", Self::reg8(&reg)));

                                    self.output.push_str(&format!(
                                        "mov byte [rax + {i}], {}\n",
                                        Self::reg8(&reg)
                                    ));

                                    self.regs.push_back(reg);
                                }

                                self.output
                                    .push_str(&format!("mov rdi, {}\nmov rsi, rax\n", str.len()));
                                self.output.push_str("call string.new\n");

                                self.output
                                    .push_str(&format!("mov qword [rbp - {offset}], rax\n"));
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
                }

                Expr::StringLiteral(str) => {
                    let offset = self.alloc_local(
                        name,
                        &&Type::Pointer(Box::new(Type::Struct {
                            name: "string".to_owned(),
                            instances: vec![
                                ("size".to_string(), Type::int),
                                ("data".to_string(), Type::Pointer(Box::new(Type::Char))),
                            ],
                        })),
                    );

                    self.output
                        .push_str(&format!("mov rdi, {}\ncall malloc\n", str.len()));

                    // rdi - size
                    // rsi - char*

                    for (i, ch) in str.chars().enumerate() {
                        let reg = self.regs.pop_front().expect("No regs");

                        self.output
                            .push_str(&format!("mov {}, '{ch}'\n", Self::reg8(&reg)));

                        self.output
                            .push_str(&format!("mov byte [rax + {i}], {}\n", Self::reg8(&reg)));

                        self.regs.push_back(reg);
                    }

                    self.output
                        .push_str(&format!("mov rdi, {}\nmov rsi, rax\n", str.len()));
                    self.output.push_str("call string.new\n");

                    self.output
                        .push_str(&format!("mov qword [rbp - {offset}], rax\n"));
                }

                Expr::StructInit { name: cls, params } => {
                    // Get the struct layout to map named parameters to field positions
                    let struct_info = self.structes.get(cls);
                    if struct_info.is_none() {
                        panic!("Struct {} not found", cls);
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
                            panic!("Field {} not found in struct {}", param_name, cls);
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

                    let ctor = format!("{cls}.new");
                    self.call_with_alignment(&ctor);

                    let struc = &Type::Pointer(Box::new(Type::Struct {
                        name: cls.to_string(),
                        instances: params
                            .iter()
                            .map(|(name, expr)| (name.clone(), expr.get_type()))
                            .collect(),
                    }));

                    let off = self.alloc_local(name, struc);

                    // println!("allocated {struct:?}");

                    self.output
                        .push_str(&format!("mov qword [rbp - {off}], rax\n"));
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

                    if let Type::Pointer(inner) = var_type {
                        if let Type::Struct {
                            name: struct_name, ..
                        } = &**inner
                        {
                            self.locals.insert(
                                name.clone(),
                                (Some(struct_name.clone()), offset, var_type.clone()),
                            );
                        }
                    }

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
                //         .structes
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
                //                     Self::reg32(&val_reg)
                //                 ));
                //             }
                //             Type::Bool => {
                //                 self.output.push_str(&format!(
                //                     "mov byte [rbp - {offset} - {field_offset}], {}\n",
                //                     Self::reg8(&val_reg)
                //                 ));
                //             }
                //             Type::Char => {
                //                 self.output.push_str(&format!(
                //                     "mov dword [rbp - {offset} - {field_offset}], {}\n",
                //                     Self::reg8(&val_reg)
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
                        .structes
                        .get(&struct_type)
                        .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                        .0;

                    // Find field offset
                    let field_offset = struct_layout
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
                    if let Some(val_reg) = self.handle_expr(value, None) {
                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], {val_reg}\n"));
                        self.regs.push_back(val_reg);
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
            Stmt::Return(None) => {}
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

    fn generate_function(&mut self, name: &str, params: Vec<(String, Type)>, body: &Vec<Stmt>) {
        self.locals.clear();
        self.stack_size = 0;

        let epilogue = format!(".Lret_{name}");
        let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];

        self.output.push_str(&format!("global {name}\n{name}:\n"));
        self.output.push_str("push rbp\nmov rbp, rsp\n");

        // Calculate total stack needed for locals + parameters
        let total_param_size: usize = params.iter().map(|(_, ty)| ty.size()).sum();
        let aligned_size = (total_param_size + 15) & !15; // align to 16 bytes
        self.stack_size = aligned_size as i32;

        self.output
            .push_str(&format!("sub rsp, {}\n", aligned_size));

        let mut offset = 0;
        for (i, (param_name, ty)) in params.iter().enumerate() {
            let size = ty.size();
            offset += size;

            // Save parameter to local stack slot
            match ty {
                Type::Char | Type::Bool => {
                    self.output.push_str(&format!(
                        "mov byte [rbp - {}], {}\n",
                        offset,
                        Self::reg8(save_regs[i])
                    ));
                }
                Type::int => {
                    self.output.push_str(&format!(
                        "mov dword [rbp - {}], {}\n",
                        offset,
                        Self::reg32(save_regs[i])
                    ));
                }
                _ => {
                    self.output
                        .push_str(&format!("mov qword [rbp - {}], {}\n", offset, save_regs[i]));
                }
            }

            let mut base_ty = ty;

            while let Type::Pointer(inside) = base_ty {
                base_ty = inside;
            }

            // Record local info
            let struct_info = match base_ty {
                // Type::Pointer(inner) => match &**inner {
                //     Type::Struct { name, .. } => Some(name.clone()),
                //     _ => None,
                // },
                Type::Struct { name, .. } => Some(name.clone()),
                _ => None,
            };

            self.locals.insert(
                param_name.clone(),
                (struct_info, offset.try_into().unwrap(), ty.clone()),
            );
        }

        // Generate body
        for stmt in body {
            self.handle_stmt_with_epilogue(stmt, &epilogue);
        }

        if name == "main" {
            self.output.push_str("xor rax, rax\n");
        }

        self.output.push_str(&format!("{epilogue}:\n"));
        self.output.push_str("mov rsp, rbp\npop rbp\nret\n");
    }

    fn generate_struct(&mut self, name: &str, instances: Vec<(String, Type)>) {
        let struct_layout = layout_fields(&instances);
        self.output
            .push_str(&format!("; ----- Layout: {name} -----\n"));
        self.output
            .push_str(&format!("%define {}_size {}\n", name, struct_layout.size));
        for fld in &struct_layout.fields {
            self.output
                .push_str(&format!("%define {}.{} {}\n", name, fld.name, fld.offset));
        }
        self.output.push('\n');

        let ctor_sym = format!("{name}.new");
        self.output
            .push_str(&format!("global {ctor_sym}\n{ctor_sym}:\n"));
        self.output.push_str("push rbp\nmov rbp, rsp\n");

        // save incoming args we will need after call to _malloc
        let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];
        let n_fields = instances.len().min(save_regs.len());

        // Ensure 16-byte stack alignment for malloc call
        // let stack_adjust = if (n_fields & 1) == 1 { 8 } else { 0 };
        // if stack_adjust > 0 {
        //     self.output
        //         .push_str(&format!("sub rsp, {}\n", stack_adjust));
        // }

        let mut stack_adj = vec![];
        for instance in instances.clone() {
            match instance.1 {
                Type::int => stack_adj.push(4),
                Type::float => stack_adj.push(4),
                Type::Char => stack_adj.push(1),
                Type::Bool => stack_adj.push(1),
                _ => stack_adj.push(8),
            }
        }

        let sum = stack_adj.iter().sum::<i32>();

        let sum = sum + (sum % 8);

        self.output.push_str(&format!("sub rsp, {sum}\n"));

        // stack_adj += 16 - (stack_adj % 16);

        // #[allow(clippy::needless_range_loop)]
        // for i in 0..n_fields {
        //     self.output.push_str(&format!(
        //         "mov qword [rbp - {}], {}\n",
        //         8 * (1 + i),
        //         save_regs[i]
        //     ));
        // }

        for (i, (_, ty)) in instances.iter().enumerate().take(n_fields) {
            let mut slot_off = 0;
            for n in 0..i + 1 {
                slot_off += stack_adj[n];
            }
            match ty {
                Type::int => {
                    self.output.push_str(&format!(
                        "mov dword [rbp - {}], {}\n",
                        slot_off,
                        Self::reg32(save_regs[i])
                    ));
                }
                Type::Char | Type::Bool => {
                    self.output.push_str(&format!(
                        "mov byte [rbp - {}], {}\n",
                        slot_off,
                        Self::reg8(save_regs[i])
                    ));
                }
                _ => {
                    self.output.push_str(&format!(
                        "mov qword [rbp - {}], {}\n",
                        slot_off, save_regs[i]
                    ));
                }
            }
        }

        self.output.push_str(&format!("mov rdi, {name}_size\n"));

        #[cfg(target_arch = "aarch64")]
        self.output.push_str("call _malloc\n");

        #[cfg(target_arch = "x86_64")]
        self.output.push_str("call malloc\n");

        // if stack_adj > 0 {
        //     self.output.push_str(&format!("add rsp, {}\n", stack_adj));
        // }

        self.output.push_str("mov rcx, rax\n");

        for (i, (_, ty)) in instances.iter().enumerate().take(n_fields) {
            let off_in_obj = struct_layout.fields[i].offset;
            let mut slot_off = 0;
            for n in 0..i + 1 {
                slot_off += stack_adj[n];
            }
            match ty {
                Type::Array(_, _) => {
                    self.output
                        .push_str(&format!("mov rax, qword [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov qword [rcx + {off_in_obj}], rax\n"));
                }
                Type::int => {
                    self.output
                        .push_str(&format!("mov eax, dword [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov dword [rcx + {off_in_obj}], eax\n"));
                }
                Type::Char | Type::Bool => {
                    self.output
                        .push_str(&format!("mov al, byte [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov byte [rcx + {off_in_obj}], al\n"));
                }
                Type::Pointer(_) => {
                    self.output
                        .push_str(&format!("mov rax, qword [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov qword [rcx + {off_in_obj}], rax\n"));
                }
                _ => {
                    self.output
                        .push_str("; TODO: unsupported field type in ctor\n");
                }
            }
        }

        self.output.push_str("mov rax, rcx\n");

        if n_fields > 0 {
            self.output.push_str(&format!("add rsp, {sum}\n"));
        }

        self.output.push_str("mov rsp, rbp\npop rbp\nret\n\n");

        // for func in funcs.clone() {
        //     if let Stmt::FunDecl {
        //         name: mname,
        //         params,
        //         body,
        //         ..
        //     } = func
        //     {
        //         let sym = format!("{name}.{mname}");
        //         // self.generate_function(&sym, params.clone(), &body);
        //         self.functions.push((sym, params.clone(), body.clone()));
        //     }
        // }

        self.structes.insert(
            name.to_string(),
            (
                struct_layout,
                Stmt::StructDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                },
            ),
        );
    }

    fn reg32(r: &str) -> &'static str {
        match r {
            "rax" => "eax",
            "rbx" => "ebx",
            "rcx" => "ecx",
            "rdx" => "edx",
            "rsi" => "esi",
            "rdi" => "edi",
            "r8" => "r8d",
            "r9" => "r9d",
            "r10" => "r10d",
            "r11" => "r11d",
            "r12" => "r12d",
            "r13" => "r13d",
            "r14" => "r14d",
            "r15" => "r15d",
            _ => "eax",
        }
    }

    fn reg8(r: &str) -> &'static str {
        match r {
            "rax" => "al",
            "rbx" => "bl",
            "rcx" => "cl",
            "rdx" => "dl",
            "rsi" => "sil",
            "rdi" => "dil",
            "r8" => "r8b",
            "r9" => "r9b",
            "r10" => "r10b",
            "r11" => "r11b",
            "r12" => "r12b",
            "r13" => "r13b",
            "r14" => "r14b",
            "r15" => "r15b",

            _ => "al",
        }
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

                // Get struct layout from structes registry
                let struct_layout = &self
                    .structes
                    .get(&struct_type)
                    .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                    .0;

                // Find field in struct layout
                let field_offset = struct_layout
                    .fields
                    .iter()
                    .find(|f| f.name == *field)
                    .map(|f| f.offset)
                    .unwrap_or_else(|| {
                        panic!("Field '{}' not found in struct '{}'", field, struct_type)
                    });

                // Get field type from struct declaration
                let field_type = {
                    let struct_decl_stmt = &self.structes.get(&struct_type).unwrap().1;
                    if let Stmt::StructDecl { instances, .. } = struct_decl_stmt {
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
                            Self::reg32(&val_reg)
                        ));
                    }
                    Type::Bool | Type::Char => {
                        self.output.push_str(&format!(
                            "mov byte [{reg} + {field_offset}], {}\n",
                            Self::reg8(&val_reg)
                        ));
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
                self.output.push_str("call _malloc\n");

                #[cfg(target_arch = "x86_64")]
                self.output.push_str("call malloc\n");

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

                    // println!("{args:?}");

                    let size = args[0].get_type().size();

                    for (i, reg) in self.regs.clone().iter().enumerate() {
                        if reg == "rax" {
                            self.regs.remove(i);
                        }
                    }

                    self.output.push_str(&format!("mov rax, {size}\n"));

                    return Some("rax".to_string());
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
                let is_defined = self.functions.iter().any(|(n, _, _)| n == name);
                let target = if is_defined {
                    name.clone()
                } else {
                    format!("{name}")
                };
                self.call_with_alignment(&target);
                for (i, reg) in self.regs.clone().iter().enumerate() {
                    if reg == "rax" {
                        self.regs.remove(i);
                    }
                }
                Some("rax".to_string())
            }
            Expr::FloatLiteral(_f) => None,
            Expr::CharLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, '{n}'\n", Self::reg8(&av_reg)));
                self.output.push_str(&format!("mov {}, '{n}'\n", av_reg));

                Some(av_reg)
            }

            Expr::BoolLiteral(b) => {
                let val = if *b { 1 } else { 0 };
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, {val}\n", Self::reg8(&av_reg)));
                self.output.push_str(&format!("mov {}, {val}\n", av_reg));
                Some(av_reg)
            }

            Expr::IntLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                // self.output
                //     .push_str(&format!("mov {}, {n}\n", Self::reg32(&av_reg)));
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
                //     .structes
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
                //     .structes
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
                    .structes
                    .get(struct_type.trim())
                    .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                    .0;

                let mut field_offset = None;
                let mut field_type = None;
                for field in &struct_layout.fields {
                    if &field.name == instance_name {
                        field_offset = Some(field.offset as i32);
                        // Get the field type from the struct definition
                        if let Some((_, struct_stmt)) = self.structes.get(struct_type.trim()) {
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

                let field_offset = field_offset.expect(&format!(
                    "Field '{}' not found in struct '{}'",
                    instance_name, struct_type
                ));

                let field_type = field_type.expect(&format!(
                    "Field type for '{}' not found in struct '{}'",
                    instance_name, struct_type
                ));

                match field_type {
                    Type::int => {
                        self.output.push_str(&format!(
                            "mov {}, dword [{ptr_reg} + {field_offset}]\n",
                            Self::reg32(&val_reg)
                        ));
                    }
                    Type::Char | Type::Bool => {
                        self.output.push_str(&format!(
                            "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                            Self::reg8(&val_reg)
                        ));
                    }
                    _ => {
                        self.output.push_str(&format!(
                            "mov {val_reg}, qword [{ptr_reg} + {field_offset}]\n"
                        ));
                    }
                }

                let _field_type = {
                    let struct_decl_stmt = &self.structes.get(&struct_type).unwrap().1;
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
                let off = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));
                let av_reg = self.regs.pop_front().expect("No registers");

                let t = &self.locals.get(name).unwrap().2;

                match t {
                    Type::int | Type::float => {
                        self.output.push_str(&format!(
                            "mov {}, dword [rbp - {off}]\n",
                            Self::reg32(&av_reg)
                        ));
                        Some(av_reg)
                    }
                    Type::Char | Type::Bool => {
                        self.output.push_str(&format!(
                            "mov {}, byte [rbp - {off}]\n",
                            Self::reg8(&av_reg)
                        ));
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
                            .push_str(&format!("mov {dst}, qword [rbp + {idx}*8 - {base_off}]\n"));
                        self.regs.push_back(idx);
                        Some(dst)
                    }
                }
            }

            Expr::StructInit { name, params } => {
                let struct_info = self.structes.get(name);

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

                let reg = self.regs.pop_front().expect("No regs");

                self.output.push_str(&format!("mov {reg}, rax\n"));

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

                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], [rbp - {var_off}]"));
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
                        // let offset = self.alloc_local(
                        //     name,
                        //     &Type::Struct {
                        //         name: "string".to_owned(),
                        //         instances: vec![
                        //             ("size".to_string(), Type::int),
                        //             ("data".to_string(), Type::Pointer(Box::new(Type::Char))),
                        //         ],
                        //     },
                        // );

                        self.output
                            .push_str(&format!("mov rdi, {}\ncall malloc\n", str.len()));

                        // rdi - size
                        // rsi - char*

                        for (i, ch) in str.chars().enumerate() {
                            let reg = self.regs.pop_front().expect("No regs");

                            self.output
                                .push_str(&format!("mov {}, '{ch}'\n", Self::reg8(&reg)));

                            self.output
                                .push_str(&format!("mov byte [rax + {i}], {}\n", Self::reg8(&reg)));

                            self.regs.push_back(reg);
                        }

                        self.output
                            .push_str(&format!("mov rdi, {}\nmov rsi, rax\n", str.len()));
                        self.output.push_str("call string.new\n");

                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], rax\n"));
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
                    //         .structes
                    //         .get(struct_type.trim())
                    //         .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                    //         .0;

                    //     let mut field_offset = None;
                    //     let mut field_type = None;
                    //     for field in &struct_layout.fields {
                    //         if &field.name == &instance_name {
                    //             field_offset = Some(field.offset as i32);
                    //             // Get the field type from the struct definition
                    //             if let Some((_, struct_stmt)) = self.structes.get(struct_type.trim()) {
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
                        let struct_type = self.locals.get(&struct_name).unwrap().0.clone().unwrap();
                        let struct_layout = &self.structes.get(&struct_type).unwrap().0;

                        let field_offset = struct_layout
                            .fields
                            .iter()
                            .find(|f| f.name == *field_name)
                            .unwrap()
                            .offset;

                        // Get value to store
                        let val_reg = self.handle_expr(value, None).expect("No value register");

                        // Store to struct field based on type
                        match value.get_type() {
                            Type::int => {
                                self.output.push_str(&format!(
                                    "mov dword [{struct_ptr_reg} + {field_offset}], {}\n",
                                    Self::reg32(&val_reg)
                                ));
                            }
                            Type::Char | Type::Bool => {
                                self.output.push_str(&format!(
                                    "mov byte [{struct_ptr_reg} + {field_offset}], {}\n",
                                    Self::reg8(&val_reg)
                                ));
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
                        .push_str(&format!("lea {addr}, [rbp + {idx_reg}*8 - {base_off}]\n"));
                    self.regs.push_back(idx_reg);
                    Some(addr)
                }
                Expr::StructInit { name, params } => {
                    // Get the struct layout to map named parameters to field positions
                    let struct_info = self.structes.get(name);

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

                    let reg = self.regs.pop_front().expect("No regs");

                    self.output.push_str(&format!("mov {reg}, rax\n"));

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
                        .structes
                        .get(&struct_type)
                        .unwrap_or_else(|| panic!("Struct layout not found for '{}'", struct_type))
                        .0;

                    let field_offset = struct_layout
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
                Expr::Variable(_, _) => {
                    let ptr = self.handle_expr(expr, None).expect("ptr reg");

                    // match ty {
                    //     Type::int | Type::float => todo!(),

                    //     Type::Char | Type::Bool => todo!(),
                    //     _ => {}
                    // }

                    self.output.push_str(&format!("mov {ptr}, qword [{ptr}]\n"));

                    Some(ptr)
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
                        .structes
                        .get(struct_type.trim())
                        .unwrap_or_else(|| panic!("Struct layout not found for '{struct_type}'"))
                        .0;

                    let mut field_offset = None;
                    let mut field_type = None;
                    for field in &struct_layout.fields {
                        if &field.name == instance_name {
                            field_offset = Some(field.offset as i32);
                            // Get the field type from the struct definition
                            if let Some((_, struct_stmt)) = self.structes.get(struct_type.trim()) {
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

                    let field_offset = field_offset.expect(&format!(
                        "Field '{}' not found in struct '{}'",
                        instance_name, struct_type
                    ));

                    let field_type = field_type.expect(&format!(
                        "Field type for '{}' not found in struct '{}'",
                        instance_name, struct_type
                    ));

                    match field_type {
                        Type::int => {
                            self.output.push_str(&format!(
                                "mov {}, dword [{ptr_reg} + {field_offset}]\n",
                                Self::reg32(&val_reg)
                            ));
                        }
                        Type::Char | Type::Bool => {
                            self.output.push_str(&format!(
                                "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                                Self::reg8(&val_reg)
                            ));
                        }
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
                    Type::int | Type::float => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output.push_str(&format!(
                                "mov dword [{ptr_reg}], {}\n",
                                Self::reg32(&val_reg)
                            ));
                            self.regs.push_back(ptr_reg);
                            self.regs.push_back(val_reg);
                        } else {
                            self.regs.push_back(ptr_reg);
                        }
                    }
                    Type::Bool | Type::Char => {
                        if let Some(val_reg) = self.handle_expr(value, None) {
                            self.output.push_str(&format!(
                                "mov byte [{ptr_reg}], {}\n",
                                Self::reg8(&val_reg)
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
                }

                None
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

                                    self.output.push_str(&format!(
                                        "mov {}, {}\n",
                                        Self::reg32(&r),
                                        n
                                    ));

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

                let lhs = self.handle_expr(left, None).unwrap();
                let rhs = self.handle_expr(right, None).unwrap();

                match op {
                    BinaryOp::Add => {
                        self.output.push_str(&format!("add {lhs}, {rhs}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Sub => {
                        self.output.push_str(&format!("sub {lhs}, {rhs}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Mul => {
                        self.output.push_str(&format!("imul {lhs}, {rhs}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Div => {
                        self.output.push_str(&format!("mov rax, {lhs}\n"));
                        self.output.push_str("xor rdx, rdx\n");
                        self.output.push_str(&format!("div {rhs}\n"));
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::Equal => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("sete al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::NotEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setne al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::Less => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setl al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::LessEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setle al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::Greater => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setg al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                    BinaryOp::GreaterEqual => {
                        self.output.push_str(&format!("cmp {lhs}, {rhs}\n"));
                        self.output.push_str("setge al\n");
                        self.output.push_str("movzx rax, al\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
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
                        Some("rax".to_string())
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
                        Some("rax".to_string())
                    }
                    BinaryOp::Mod => {
                        self.output.push_str(&format!("mov rax, {lhs}\n"));
                        self.output.push_str("xor rdx, rdx\n");
                        self.output.push_str(&format!("div {rhs}\n"));
                        self.output.push_str("mov rax, rdx\n");
                        self.regs.push_back(lhs);
                        self.regs.push_back(rhs);
                        Some("rax".to_string())
                    }
                }
            }
            _ => None,
        }
    }
}
