use crate::lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp};
use std::collections::{HashMap, HashSet, VecDeque};

type Functions = Vec<(String, Vec<(String, Type)>, Vec<Stmt>)>;

pub struct CodeGen {
    output: String,
    regs: VecDeque<String>,
    _fp_regs: VecDeque<String>,
    _jmp_count: u32,
    functions: Functions,
    locals: HashMap<String, i32>,
    stack_size: i32,
    externs: HashSet<String>,
    classes: HashMap<String, (ClassLayout, Stmt)>, // stmt = classdec
}
#[inline]
fn align_up(x: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two()); // FIX: check `a`, not `x`
    (x + a - 1) & !(a - 1)
}

fn gpr_name(i: usize, ty: &Type) -> &'static str {
    match ty {
        Type::int => match i {
            0 => "edi",
            1 => "esi",
            2 => "edx",
            3 => "ecx",
            4 => "r8d",
            5 => "r9d",
            _ => unreachable!(),
        },
        Type::Pointer(_) => match i {
            0 => "rdi",
            1 => "rsi",
            2 => "rdx",
            3 => "rcx",
            4 => "r8",
            5 => "r9",
            _ => unreachable!(),
        },
        Type::Char | Type::Bool => match i {
            0 => "dil",
            1 => "sil",
            2 => "dl",
            3 => "cl",
            4 => "r8b",
            5 => "r9b",
            _ => unreachable!(),
        },
        _ => "rax", // fallback
    }
}

fn size_align_of(ty: &Type) -> (usize, usize) {
    match ty {
        Type::Char | Type::Bool => (1, 1),
        Type::int => (4, 4),
        Type::float => (4, 4),
        Type::Pointer(_) => (8, 8),
        Type::Array(elem, Some(n)) => {
            let (es, ea) = size_align_of(elem);
            (es * *n, ea)
        }
        Type::Array(_, None) => (16, 8),
        Type::Class(instances) => layout_of_class(instances),
        Type::Void | Type::Unknown => (0, 1),
        _ => (0, 1),
    }
}

fn layout_of_class(instances: &[Type]) -> (usize, usize) {
    // Empty class: size 0, align 1 (you can choose to make size at least 1 if you prefer)
    if instances.is_empty() {
        return (0, 1);
    }

    let mut off = 0usize;
    let mut max_a = 1usize;

    for ty in instances {
        let (sz, al) = size_align_of(ty);
        // Treat unsized as error if you don't want to allow it here:
        // if sz == 0 { return (0, 1); } // or panic/Result
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
struct ClassLayout {
    size: usize,
    align: usize,
    fields: Vec<FieldLayout>,
}

fn layout_fields(fields: &[(String, Type)]) -> ClassLayout {
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
    ClassLayout {
        size,
        align: max_a,
        fields: out,
    }
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
            classes: HashMap::new(),
        };

        #[cfg(target_arch = "aarch64")]
        code.output.push_str("extern _malloc\n");

        #[cfg(target_arch = "x86_64")]
        code.output.push_str("extern malloc\n");

        code.output.push_str("global _start\n_start:\n");
        code.output.push_str("call main\n"); // main(): returns int in rax
        code.output.push_str("mov rbx, rax\n"); // keep a copy in RBX
        // macOS x86_64 exit(status): rax=0x2000001, rdi=status
        code.output.push_str("mov rdi, rax\n");
        #[cfg(target_arch = "aarch64")]
        code.output.push_str("mov rax, 0x2000001\n");

        #[cfg(target_arch = "x86_64")]
        code.output.push_str("mov rax, 1\n");

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

            Stmt::ClassDecl {
                name,
                instances,
                funcs,
            } => {
                self.generate_class(name, instances.to_vec(), funcs.to_vec());
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

    fn generate_function(&mut self, name: &str, params: &Vec<(String, Type)>, body: &Vec<Stmt>) {
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

    fn generate_class(&mut self, name: &str, instances: Vec<(String, Type)>, functions: Vec<Stmt>) {
        let cname = name;

        // 1) Layout defines
        let class_layout = layout_fields(&instances);
        self.output
            .push_str(&format!("; ----- Layout: {cname} -----\n"));
        self.output
            .push_str(&format!("%define {}_size {}\n", cname, class_layout.size));
        for fld in &class_layout.fields {
            self.output
                .push_str(&format!("%define {}_{} {}\n", cname, fld.name, fld.offset));
        }
        self.output.push('\n');

        let arg_regs64 = self.regs.clone();

        let ctor_sym = format!("{cname}_new");
        self.output
            .push_str(&format!("global {ctor_sym}\n{ctor_sym}:\n"));
        self.output.push_str("push rbp\nmov rbp, rsp\n");

        for _ in instances.iter() {
            let reg = self.regs.pop_front().unwrap(); //gpr_name(i, ty);
            self.output.push_str(&format!("push {reg}\n"));
            self.regs.push_back(reg);
        }

        // Save arg registers we’ll overwrite by calling malloc.
        let n_fields = instances.len();
        let n_gpr = n_fields.min(arg_regs64.len());

        let _ = (0..n_gpr).map(|i| {
            self.output.push_str(&format!("push {}\n", arg_regs64[i]));
        });

        if n_gpr % 2 == 1 {
            self.output.push_str("sub rsp, 8\n");
        }

        self.output.push_str(&format!("mov rdi, {cname}_size\n"));

        #[cfg(target_arch = "aarch64")]
        self.output.push_str("call _malloc\n");

        #[cfg(target_arch = "x86_64")]
        self.output.push_str("call malloc\n");

        if n_gpr % 2 == 1 {
            self.output.push_str("add rsp, 8\n");
        }

        for (i, (_, ty)) in instances.iter().enumerate().take(n_gpr) {
            let off_in_obj = class_layout.fields[i].offset;
            let slot_off = (i + 1) * 8; // [rbp - slot_off]
            match ty {
                Type::int => {
                    self.output
                        .push_str(&format!("mov eax, dword [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov dword [rax + {off_in_obj}], eax\n"));
                }
                Type::Char | Type::Bool => {
                    self.output
                        .push_str(&format!("mov al, byte [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov byte [rax + {off_in_obj}], al\n"));
                }
                Type::Pointer(_) => {
                    self.output
                        .push_str(&format!("mov rcx, qword [rbp - {slot_off}]\n"));
                    self.output
                        .push_str(&format!("mov qword [rax + {off_in_obj}], rcx\n"));
                }
                _ => {
                    self.output
                        .push_str("; TODO: unsupported field type in ctor\n");
                }
            }
        }

        // Pop saved registers (not strictly required since we’re returning)
        for _ in 0..n_gpr {
            self.output.push_str("add rsp, 8\n");
        }

        self.output.push_str("mov rsp, rbp\npop rbp\nret\n\n");

        // 3) Generate class methods (mangled: Class_method) and mark as defined
        for func in functions.clone() {
            if let Stmt::FunDecl {
                name: mname,
                params,
                body,
                ..
            } = func
            {
                let sym = format!("{}_{}", cname, mname);
                self.generate_function(&sym, &params, &body);
                self.functions.push((sym, params.clone(), body.clone()));
            }
        }

        // 4) Remember the class
        self.classes.insert(
            name.to_string(),
            (
                class_layout,
                Stmt::ClassDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    funcs: functions,
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
            Expr::CharLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers available");

                self.output.push_str(&format!("mov {av_reg}, '{n}'\n"));

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
                        let off = base_off - *n * 8;
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
