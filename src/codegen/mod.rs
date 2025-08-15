use crate::lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp};
use std::collections::{HashMap, HashSet, VecDeque};

type Functions = Vec<(String, Vec<(String, Type)>, Vec<Stmt>)>;

pub struct CodeGen {
    output: String,
    imports: Vec<String>,
    regs: VecDeque<String>,
    _fp_regs: VecDeque<String>,
    _jmp_count: u32,
    functions: Functions,
    locals: HashMap<String, (Option<String>, i32)>,
    stack_size: i32,
    externs: HashSet<String>,
    classes: HashMap<String, (ClassLayout, Stmt)>,
}

#[inline]
fn align_up(x: usize, a: usize) -> usize {
    debug_assert!(a.is_power_of_two());
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
        Type::Class { instances, .. } => layout_of_class(
            &instances
                .iter()
                .map(|(_, ty)| ty.clone())
                .collect::<Vec<Type>>(),
        ),
        Type::Void | Type::Unknown => (0, 1),
        _ => (0, 1),
    }
}

fn layout_of_class(instances: &[Type]) -> (usize, usize) {
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
    // asm      import paths
    pub fn generate(stmts: &Vec<Stmt>) -> (String, Vec<String>) {
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
            classes: HashMap::new(),
            imports: Vec::new(),
        };

        code.output.push_str("extern _malloc\n");

        code.output.push_str("global _start\n_start:\n");
        code.output.push_str("call main\n");
        code.output.push_str("mov rbx, rax\n");
        code.output.push_str("mov rdi, rax\n");
        code.output.push_str("mov rax, 0x2000001\n");
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
            if let Stmt::ClassDecl {
                name,
                instances,
                funcs,
            } = stmt
            {
                code.generate_class(name, instances.clone(), funcs.to_vec());
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
        for ext in code.externs.difference(&defined) {
            header.push_str(&format!("extern _{ext}\n"));
        }

        (format!("{header}{}", code.output), code.imports)
    }

    fn alloc_local(&mut self, name: &str, class_name: Option<String>, _ty: &Type) -> i32 {
        self.stack_size += 8;
        self.output.push_str("sub rsp, 8\n");
        let offset = self.stack_size;
        self.locals.insert(name.to_string(), (class_name, offset));
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
            Stmt::AtDecl(decl, param) => if decl.as_str() == "import" {
                let param = param
                    .clone()
                    .unwrap_or_else(|| panic!("Unable to locate import"));
                self.imports.push(param);
            },
            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => match value {
                Expr::Array(elements, _ty) => {
                    let count = elements.len();
                    let total_size = (count as i32) * 8;
                    self.stack_size += total_size;
                    self.output.push_str(&format!("sub rsp, {total_size}\n"));
                    let base_offset = self.stack_size;
                    self.locals.insert(name.clone(), (None, base_offset));

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
                Expr::ClassInit { name: cls, params } => {
                    // Get the class layout to map named parameters to field positions
                    let class_info = self.classes.get(cls);
                    if class_info.is_none() {
                        panic!("Class {} not found", cls);
                    }
                    let (class_layout, _) = class_info.unwrap();
                    
                    // Clone the field names and positions to avoid borrowing issues
                    let field_positions: HashMap<String, usize> = class_layout.fields.iter()
                        .enumerate()
                        .map(|(i, field)| (field.name.clone(), i))
                        .collect();
                    let field_count = class_layout.fields.len();
                    
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
                            panic!("Field {} not found in class {}", param_name, cls);
                        }
                    }
                    
                    // Filter out empty strings and get the actual argument values
                    let arg_vals: Vec<String> = ordered_args.into_iter()
                        .filter(|s| !s.is_empty())
                        .collect();

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

                    let off = self.alloc_local(
                        name,
                        Some(cls.to_string()),
                        &Type::Pointer(Box::new(Type::Class {
                            name: cls.to_string(),
                            instances: params
                                .iter()
                                .map(|(name, expr)| (name.clone(), expr.get_type()))
                                .collect(),
                        })),
                    );

                    self.output
                        .push_str(&format!("mov qword [rbp - {off}], rax\n"));
                }
                _ => {
                    let offset = if self.local_offset(name).is_none() {
                        self.alloc_local(name, None, var_type)
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

            // Stmt::ClassDecl {
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

        #[allow(clippy::needless_range_loop)]
        for (i, param) in params.iter().enumerate() {
            let off = match &param.1 {
                Type::Class { name, .. } => {
                    self.alloc_local(&param.0, Some(name.to_string()), &param.1)
                }
                _ => self.alloc_local(&param.0, None, &param.1),
            };

            self.output
                .push_str(&format!("mov {}, qword [rbp - {off}]\n", save_regs[i]));
        }

        for stmt in body {
            self.handle_stmt_with_epilogue(stmt, &epilogue);
        }

        if name == "main" {
            self.output.push_str("xor rax, rax\n");
        }

        self.output.push_str(&format!("{epilogue}:\n"));
        self.output.push_str("mov rsp, rbp\npop rbp\nret\n");
    }

    fn generate_class(&mut self, name: &str, instances: Vec<(String, Type)>, funcs: Vec<Stmt>) {
        let class_layout = layout_fields(&instances);
        self.output
            .push_str(&format!("; ----- Layout: {name} -----\n"));
        self.output
            .push_str(&format!("%define {}_size {}\n", name, class_layout.size));
        for fld in &class_layout.fields {
            self.output
                .push_str(&format!("%define {}_{} {}\n", name, fld.name, fld.offset));
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
        let stack_adjust = if (n_fields & 1) == 1 { 8 } else { 0 };
        if stack_adjust > 0 {
            self.output.push_str(&format!("sub rsp, {}\n", stack_adjust));
        }

        #[allow(clippy::needless_range_loop)]
        for i in 0..n_fields {
            self.output.push_str(&format!("push {}\n", save_regs[i]));
        }

        self.output.push_str(&format!("mov rdi, {name}_size\n"));

        #[cfg(target_arch = "aarch64")]
        self.output.push_str("call _malloc\n");

        #[cfg(target_arch = "x86_64")]
        self.output.push_str("call _malloc\n");

        if stack_adjust > 0 {
            self.output.push_str(&format!("add rsp, {}\n", stack_adjust));
        }

        self.output.push_str("mov rcx, rax\n");

        for (i, (_, ty)) in instances.iter().enumerate().take(n_fields) {
            let off_in_obj = class_layout.fields[i].offset;
            let slot_off = (i + 1) * 8;
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
            self.output
                .push_str(&format!("add rsp, {}\n", n_fields * 8));
        }

        self.output.push_str("mov rsp, rbp\npop rbp\nret\n\n");

        for func in funcs.clone() {
            if let Stmt::FunDecl {
                name: mname,
                params,
                body,
                ..
            } = func
            {
                let sym = format!("{name}.{mname}");
                // self.generate_function(&sym, params.clone(), &body);
                self.functions.push((sym, params.clone(), body.clone()));
            }
        }

        self.classes.insert(
            name.to_string(),
            (
                class_layout,
                Stmt::ClassDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    funcs: funcs.to_vec(),
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
                            .push_str(&format!("mov {} , {}\n", abi_regs[i], t));
                    }
                }
                for t in temps {
                    self.regs.push_back(t);
                }
                self.externs.insert(name.clone());
                let is_defined = self.functions.iter().any(|(n, _, _)| n == name);
                let target = if is_defined {
                    name.clone()
                } else {
                    format!("_{name}")
                };
                self.call_with_alignment(&target);
                Some("rax".to_string())
            }
            Expr::FloatLiteral(_f) => None,
            Expr::BoolLiteral(n) => {
                let val = if *n { 1 } else { 0 };
                let av_reg = self.regs.pop_front().expect("No registers");
                self.output.push_str(&format!("mov {av_reg}, {val}\n"));
                Some(av_reg.to_string())
            }
            Expr::IntLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                self.output.push_str(&format!("mov {av_reg}, {n}\n"));
                Some(av_reg.to_string())
            }
            Expr::CharLiteral(n) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                self.output.push_str(&format!("mov {av_reg}, '{n}'\n"));
                Some(av_reg.to_string())
            }
            Expr::InstanceVar(class_name, instance_name) => {
                let av_reg = self.regs.pop_front().expect("No registers");
                let mut off = self
                    .local_offset(class_name)
                    .expect("Error reading class name");

                let class_ptr_reg = self
                    .handle_expr(&Expr::Variable(class_name.to_string()), None)
                    .unwrap_or_else(|| panic!("Could not locate: '{class_name}'"));

                let class_type = self
                    .locals
                    .get(class_name)
                    .unwrap_or_else(|| {
                        panic!("Error parsing class type for variable: {class_name}")
                    })
                    .0
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("Error parsing class type for variable: {class_name}")
                    });

                let class_layout = &self
                    .classes
                    .get(class_type.trim())
                    .unwrap_or_else(|| {
                        panic!("Error parsing class type for variable: {class_name}")
                    })
                    .0;

                for field in &class_layout.fields {
                    if &field.name == instance_name {
                        off += field.offset as i32;
                    }
                }

                self.output
                    .push_str(&format!("mov {av_reg}, qword [{class_ptr_reg} - {off}]\n"));

                self.regs.push_front(class_ptr_reg);

                Some(av_reg)
            }
            Expr::Variable(name) => {
                let off = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));
                let av_reg = self.regs.pop_front().expect("No registers");
                self.output
                    .push_str(&format!("mov {av_reg}, qword [rbp - {off}]\n"));
                Some(av_reg)
            }
            Expr::ArrayAccess { array, index } => {
                let (_name, base_off) = match &**array {
                    Expr::Variable(n) => {
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

            Expr::Assign { name, value } => {
                let offset = self
                    .local_offset(name)
                    .unwrap_or_else(|| panic!("Unknown var '{name}'"));
                match **value {
                    Expr::IntLiteral(n) => {
                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], {n}\n"));
                    }
                    Expr::BoolLiteral(b) => {
                        let val = if b { 1 } else { 0 };
                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], {val}\n"));
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
                Expr::Variable(var_name) => {
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
                            Expr::Variable(n) => n,
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
                _ => {
                    let ptr = self.handle_expr(expr, None).expect("ptr reg");
                    self.output.push_str(&format!("mov {ptr}, qword [{ptr}]\n"));
                    Some(ptr)
                }
            },

            Expr::DerefAssign { target, value } => {
                let ptr_reg = self.handle_expr(target, None).expect("ptr reg");
                if let Some(val_reg) = self.handle_expr(value, None) {
                    self.output
                        .push_str(&format!("mov qword [{ptr_reg}], {val_reg}\n"));
                    self.regs.push_back(ptr_reg);
                    self.regs.push_back(val_reg);
                } else {
                    self.regs.push_back(ptr_reg);
                }
                None
            }

            Expr::Binary {
                left, op, right, ..
            } => {
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
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
