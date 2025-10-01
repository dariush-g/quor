use crate::{
    analyzer::base_type,
    lexer::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
};

// add @trust_ret for inline asm functions so that it trusts that the correct type and value will
// be returned from the function
//
// maybe @undef_params for functions with an undefined number of paramaters:
// @undef_params def function(argc: int, args: void*) {} or something like that
//

use std::{
    collections::{HashMap, HashSet, VecDeque}, fs
};

type Functions = Vec<(String, Vec<(String, Type)>, Vec<Stmt>, Vec<String>)>;

pub struct CodeGen {
    output: String,
    // imports: Vec<String>,
    globals: HashMap<String, Type>,
    regs: VecDeque<String>,
    fp_regs: VecDeque<String>,
    fp_count: u32,
    _jmp_count: u32,
    functions: Functions,
    locals: HashMap<String, (Option<String>, i32, Type)>,
    stack_size: i32,
    externs: HashSet<String>,
    structures: HashMap<String, (StructLayout, Stmt)>,
    rodata: String,
    bss: String,
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
        Type::Pointer(_) | Type::Long => (8, 8),
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
    _size: usize,
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
        _size: size,
        _align: max_a,
        fields: out,
    }
}

impl CodeGen {
    fn const_eval_int(expr: &Expr) -> Option<i64> {
        match expr {
            Expr::IntLiteral(n) => Some(*n as i64),
            Expr::LongLiteral(n) => Some(*n),
            Expr::Unary {
                op: UnaryOp::Negate,
                expr,
                ..
            } => Self::const_eval_int(expr).map(|v| -v),
            Expr::Binary {
                left, op, right, ..
            } => {
                let l = Self::const_eval_int(left)?;
                let r = Self::const_eval_int(right)?;
                match op {
                    BinaryOp::Add => Some(l + r),
                    BinaryOp::Sub => Some(l - r),
                    BinaryOp::Mul => Some(l * r),
                    BinaryOp::Div => Some(l / r),
                    BinaryOp::Mod => Some(l % r),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    // asm      import paths
    pub fn generate(stmts: &Vec<Stmt>) -> String {
        let mut code = CodeGen {
            globals: HashMap::new(),
            fp_count: 0,
            output: String::new(),
            regs: VecDeque::from(vec![
                "rcx".to_string(),
                "rdx".to_string(),
                "rbx".to_string(),
                "r8".to_string(),
                "r9".to_string(),
                "r10".to_string(),
                "r11".to_string(),
                "r12".to_string(),
                "r13".to_string(),
                "r14".to_string(),
                "r15".to_string(),
            ]),
            fp_regs: VecDeque::from(vec![
                "xmm0".to_string(),
                "xmm1".to_string(),
                "xmm2".to_string(),
                "xmm3".to_string(),
                "xmm4".to_string(),
                "xmm5".to_string(),
                "xmm6".to_string(),
                "xmm7".to_string(),
                "xmm8".to_string(),
                "xmm9".to_string(),
                "xmm10".to_string(),
                "xmm11".to_string(),
                "xmm12".to_string(),
                "xmm13".to_string(),
                "xmm14".to_string(),
                "xmm15".to_string(),
            ]),
            _jmp_count: 0,
            functions: vec![],
            locals: HashMap::new(),
            stack_size: 0,
            externs: HashSet::new(),
            structures: HashMap::new(),
            bss: String::new(),
            rodata: String::new(),
        };

        // #[cfg(target_arch = "aarch64")]
        // code.output.push_str("extern _malloc\n");
        // #[cfg(target_arch = "x86_64")]
        // code.output.push_str("extern malloc\n");

        // code.output.push_str("global _start\n_start:\n");

        // code.output.push_str("call main\n");
        // code.output.push_str("mov rbx, rax\n");

        // #[cfg(target_arch = "aarch64")]
        // code.output.push_str("mov rax, 0x2000001\n");
        // #[cfg(target_arch = "x86_64")]
        // code.output.push_str("mov rax, 60\n");
        // code.output.push_str("syscall\n");

        let mut has_main = false;

        for stmt in stmts {
            if let Stmt::AtDecl(decl, param, val, _) = stmt {
                if decl.as_str() == "define" || decl.as_str() == "defines" {
                    let param = param.clone().unwrap();
                    code.output.push_str("section .data\n");
                    if let Some(val) = val {
                        match val {
                            Expr::IntLiteral(n) => {
                                code.output.push_str(&format!("{param}: dd {n}\n"));
                                code.globals.insert(param.clone(), Type::int);
                            }
                            Expr::LongLiteral(n) => {
                                code.output.push_str(&format!("{param}: dq {n}\n"));
                                code.globals.insert(param.clone(), Type::Long);
                            }
                            Expr::FloatLiteral(n) => {
                                code.output.push_str(&format!("{param}: dd {n}\n"));
                                code.globals.insert(param.clone(), Type::float);
                            }
                            Expr::BoolLiteral(n) => {
                                code.output.push_str(&format!("{param}: db {}\n", *n as u8));
                                code.globals.insert(param.clone(), Type::Bool);
                            }
                            Expr::CharLiteral(n) => {
                                code.output.push_str(&format!("{param}: db {n}\n"));
                                code.globals.insert(param.clone(), Type::Char);
                            }
                            Expr::StringLiteral(n) => {
                                code.output.push_str(&format!("{param}: db \"{n}\",0\n"));
                                code.globals
                                    .insert(param.clone(), Type::Pointer(Box::new(Type::Char)));
                            }
                            other => {
                                if let Some(v) = CodeGen::const_eval_int(other) {
                                    code.output.push_str(&format!("{param}: dd {v}\n"));
                                    code.globals.insert(param.clone(), Type::int);
                                } else {
                                    // Fallback: reserve 4 bytes and mark as int
                                    code.output.push_str(&format!("{param}: dd 0\n"));
                                    code.globals.insert(param.clone(), Type::int);
                                }
                            }
                        }
                    }
                    code.output.push_str("section .text\n");
                } 
            }
            if let Stmt::FunDecl {
                name,
                params,
                body,
                attributes,
                ..
            } = stmt
            {
                if name == "main" {
                    if params.len() > 0 {
                        if params[0].1 == Type::int {
                            if let Type::Pointer(boxed_ty) = &params[1].1 {
                                if let Type::Pointer(inside) = *boxed_ty.clone() {
                                    if *inside == Type::Char {
                                        code.generate_function(
                                            "main",
                                            params.clone(),
                                            body,
                                            attributes,
                                        );
                                        has_main = true;
                                    }
                                }
                            }
                        }
                    } else {
                        code.generate_function("main", vec![], body, attributes);
                        has_main = true;
                    }
                } else {
                    code.functions.push((
                        name.clone(),
                        params.clone(),
                        body.clone(),
                        attributes.clone(),
                    ));
                }
            }
            if let Stmt::StructDecl {
                name,
                instances,
                union,
            } = stmt
            {
                code.generate_struct(name, instances.clone(), *union);
            }
        }

        if !has_main {
            panic!("No main function found");
        }

        let functions = &code.functions.clone();
        for (name, params, body, attributes) in functions {
            code.generate_function(name, params.clone(), body, attributes);
        }

        for stmt in stmts {
            if !matches!(stmt, Stmt::FunDecl { .. }) {
                code.handle_stmt(stmt);
            }
        }

        let defined: HashSet<String> = code
            .functions
            .iter()
            .map(|(n, _, _, _)| n.clone())
            .collect();
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

        let mut print = fs::read_to_string(format!("{manifest_dir}/stdlib/io.asm"))
            .unwrap_or_else(|_| panic!("Error importing io"));

        print.push('\n');

        code.output.push_str(&print);

        let mut mem = fs::read_to_string(format!("{manifest_dir}/stdlib/mem.asm"))
            .unwrap_or_else(|_| panic!("Error importing mem"));

        mem.push('\n');

        code.output.push_str(&mem);

        let bss = &format!("section .bss\n{}\n", &code.bss);
        let ro = &format!("section .rodata\n{}\nsection .text\n", &code.rodata);

        let hd = &format!("{bss}{ro}");

        code.output.insert_str(0, hd);


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
        let need_pad = (slots & 1) != 0; // odd -> needs pad
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
            Stmt::AtDecl(decl, params, _, _) => {
                if decl.as_str() == "__asm__" || decl.as_str() == "_asm_" || decl.as_str() == "asm"
                {
                    self.output
                        .push_str(&params.clone().unwrap_or("".to_string()));
                }
                if decl.as_str() == "__asm_ro__" || decl.as_str() == "_asm_ro_" || decl.as_str() == "asm_ro" {
                    self.rodata.push_str(&params.clone().unwrap());
                }
                if decl.as_str() == "__asm_bss__" || decl.as_str() == "_asm_bss_" || decl.as_str() == "asm_bss" {
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
                    let reg = self.handle_expr(&expr, None).unwrap();
                    let off = self.alloc_local(name, target_type);

                    match target_type {
                        Type::int => self
                            .output
                            .push_str(&format!("mov dword [rbp - {off}], {}\n", Self::reg32(&reg))),

                        Type::Bool | Type::Char => self
                            .output
                            .push_str(&format!("mov byte [rbp - {off}], {}\n", Self::reg8(&reg))),

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
                                if let Some(val_reg) = self.handle_expr(&element, None) {
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

                    if let Stmt::StructDecl { union, .. } = struct_stmt {
                        if *union {
                            field_offset = 0;
                        }
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
                                        Self::reg32(&val_reg)
                                    )),
                                    Type::Char | Type::Bool => self.output.push_str(&format!(
                                        "mov byte [rbp - {offset}], {}\n",
                                        Self::reg8(&val_reg)
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

    fn generate_function(
        &mut self,
        name: &str,
        params: Vec<(String, Type)>,
        body: &Vec<Stmt>,
        attributes: &Vec<String>,
    ) {
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

        // Check if function has @trust_ret attribute
        let _has_trust_ret = attributes.contains(&"trust_ret".to_string());

        // Generate body
        for stmt in body {
            self.handle_stmt_with_epilogue(stmt, &epilogue);
        }

        // Only add default return value if not @trust_ret

       
        //if !has_trust_ret {
            self.output.push_str(&format!("{epilogue}:\n"));
            self.output.push_str("mov rsp, rbp\npop rbp\nret\n");
        //}
    }

    // fn generate_struct(&mut self, name: &str, instances: Vec<(String, Type)>, union: bool) {
    //     if !union {
    //         let struct_layout = layout_fields(&instances);
    //         self.output
    //             .push_str(&format!("; ----- Layout: {name} -----\n"));
    //         self.output
    //             .push_str(&format!("%define {}_size {}\n", name, struct_layout.size));
    //         for fld in &struct_layout.fields {
    //             self.output
    //                 .push_str(&format!("%define {}.{} {}\n", name, fld.name, fld.offset));
    //         }
    //         self.output.push('\n');

    //         let ctor_sym = format!("{name}.new");
    //         self.output
    //             .push_str(&format!("global {ctor_sym}\n{ctor_sym}:\n"));
    //         self.output.push_str("push rbp\nmov rbp, rsp\n");

    //         // save incoming args we will need after call to _malloc
    //         let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];
    //         let save_fp = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4"];

    //         let mut fpc = 0;

    //         let n_fields = instances.len().min(save_regs.len());

    //         // Ensure 16-byte stack alignment for malloc call
    //         // let stack_adjust = if (n_fields & 1) == 1 { 8 } else { 0 };
    //         // if stack_adjust > 0 {
    //         //     self.output
    //         //         .push_str(&format!("sub rsp, {}\n", stack_adjust));
    //         // }

    //         let mut stack_adj = vec![];
    //         for instance in instances.clone() {
    //             match instance.1 {
    //                 Type::int => stack_adj.push(4),
    //                 Type::float => stack_adj.push(4),
    //                 Type::Char => stack_adj.push(1),
    //                 Type::Bool => stack_adj.push(1),
    //                 _ => stack_adj.push(8),
    //             }
    //         }

    //         let sum = stack_adj.iter().sum::<i32>();

    //         let sum = sum + (sum % 8);

    //         self.output.push_str(&format!("sub rsp, {sum}\n"));

    //         // stack_adj += 16 - (stack_adj % 16);

    //         // #[allow(clippy::needless_range_loop)]
    //         // for i in 0..n_fields {
    //         //     self.output.push_str(&format!(
    //         //         "mov qword [rbp - {}], {}\n",
    //         //         8 * (1 + i),
    //         //         save_regs[i]
    //         //     ));
    //         // }

    //         for (i, (_, ty)) in instances.iter().enumerate().take(n_fields) {
    //             let mut slot_off = 0;
    //             for n in 0..i + 1 {
    //                 slot_off += stack_adj[n];
    //             }
    //             match ty {
    //                 Type::int => {
    //                     self.output.push_str(&format!(
    //                         "mov dword [rbp - {}], {}\n",
    //                         slot_off,
    //                         Self::reg32(save_regs[i])
    //                     ));
    //                 }
    //                 Type::Char | Type::Bool => {
    //                     self.output.push_str(&format!(
    //                         "mov byte [rbp - {}], {}\n",
    //                         slot_off,
    //                         Self::reg8(save_regs[i])
    //                     ));
    //                 }
    //                 Type::float => {
    //                     self.output
    //                         .push_str(&format!("movss [rbp - {}], {}\n", slot_off, save_fp[fpc]));
    //                     fpc += 1;
    //                 }
    //                 _ => {
    //                     self.output.push_str(&format!(
    //                         "mov qword [rbp - {}], {}\n",
    //                         slot_off, save_regs[i]
    //                     ));
    //                 }
    //             }
    //         }

    //         self.output.push_str(&format!("mov rdi, {name}_size\n"));

    //         #[cfg(target_arch = "aarch64")]
    //         self.output.push_str("call _malloc\n");

    //         #[cfg(target_arch = "x86_64")]
    //         self.output.push_str("call malloc\n");

    //         // if stack_adj > 0 {
    //         //     self.output.push_str(&format!("add rsp, {}\n", stack_adj));
    //         // }

    //         self.output.push_str("mov rcx, rax\n");

    //         for (i, (_, ty)) in instances.iter().enumerate().take(n_fields) {
    //             let off_in_obj = struct_layout.fields[i].offset;
    //             let mut slot_off = 0;
    //             for n in 0..i + 1 {
    //                 slot_off += stack_adj[n];
    //             }
    //             match ty {
    //                 Type::Array(_, _) => {
    //                     self.output
    //                         .push_str(&format!("mov rax, qword [rbp - {slot_off}]\n"));
    //                     self.output
    //                         .push_str(&format!("mov qword [rcx + {off_in_obj}], rax\n"));
    //                 }
    //                 Type::int => {
    //                     self.output
    //                         .push_str(&format!("mov eax, dword [rbp - {slot_off}]\n"));
    //                     self.output
    //                         .push_str(&format!("mov dword [rcx + {off_in_obj}], eax\n"));
    //                 }
    //                 Type::Char | Type::Bool => {
    //                     self.output
    //                         .push_str(&format!("mov al, byte [rbp - {slot_off}]\n"));
    //                     self.output
    //                         .push_str(&format!("mov byte [rcx + {off_in_obj}], al\n"));
    //                 }
    //                 Type::Pointer(_) | Type::Long => {
    //                     self.output
    //                         .push_str(&format!("mov rax, qword [rbp - {slot_off}]\n"));
    //                     self.output
    //                         .push_str(&format!("mov qword [rcx + {off_in_obj}], rax\n"));
    //                 }
    //                 _ => {
    //                     self.output
    //                         .push_str("; TODO: unsupported field type in ctor\n");
    //                 }
    //             }
    //         }

    //         self.output.push_str("mov rax, rcx\n");

    //         if n_fields > 0 {
    //             self.output.push_str(&format!("add rsp, {sum}\n"));
    //         }

    //         self.output.push_str("mov rsp, rbp\npop rbp\nret\n\n");

    //         // for func in funcs.clone() {
    //         //     if let Stmt::FunDecl {
    //         //         name: mname,
    //         //         params,
    //         //         body,
    //         //         ..
    //         //     } = func
    //         //     {
    //         //         let sym = format!("{name}.{mname}");
    //         //         // self.generate_function(&sym, params.clone(), &body);
    //         //         self.functions.push((sym, params.clone(), body.clone()));
    //         //     }
    //         // }

    //         self.structures.insert(
    //             name.to_string(),
    //             (
    //                 struct_layout,
    //                 Stmt::StructDecl {
    //                     name: name.to_string(),
    //                     instances: instances.to_vec(),
    //                     union: union,
    //                 },
    //             ),
    //         );
    //     } else {
    //         for instance in instances.clone() {
    //             let struct_layout = layout_fields(&instances);
    //             self.output
    //                 .push_str(&format!("; ----- Layout: Union {name} -----\n"));
    //             self.output
    //                 .push_str(&format!("%define {}_size {}\n", name, struct_layout.size));
    //             for fld in &struct_layout.fields {
    //                 self.output
    //                     .push_str(&format!("%define {}.{} {}\n", name, fld.name, fld.offset));
    //             }
    //             self.output.push('\n');

    //             let ctor_sym = format!("{name}.new");
    //             self.output
    //                 .push_str(&format!("global {ctor_sym}\n{ctor_sym}:\n"));
    //             self.output.push_str("push rbp\nmov rbp, rsp\n");

    //             // let save_reg = "rdi";
    //             // let save_fp = "xmm0";

    //             let sizes = instances
    //                 .iter()
    //                 .map(|instance| instance.1.size())
    //                 .collect::<Vec<usize>>();
    //             let max = sizes.iter().max();

    //             let alignment = 16;
    //             let size = max.unwrap();

    //             self.output.push_str(&format!(
    //                 "malloc {}\n",
    //                 (size + alignment - 1) / alignment * alignment
    //             ));

    //             self.output.push_str(&format!("mov rdi, [rax]\n"));

    //             self.structures.insert(
    //                 name.to_string(),
    //                 (
    //                     struct_layout,
    //                     Stmt::StructDecl {
    //                         name: name.to_string(),
    //                         instances: instances.to_vec(),
    //                         union: union,
    //                     },
    //                 ),
    //             );
    //         }
    //     }
    // }

    fn _generate_stack_union_inline(
        &mut self,
        name: &str,
        instances: Vec<(String, Type)>,
        i: usize,
        off: usize,
    ) -> String {
        let mut size = 0;
        let mut output = String::new();

        for instance in &instances {
            size = instance.1.size().max(size);
        }

        output.push_str(&format!("; ----- Inline stack struct: {} -----\n", name));
        output.push_str(&format!("%define {}_size {}\n", name, size));

        output.push('\n');

        // Compute stack size
        let stack_offset = size;

        let aligned_size = (stack_offset + 7) & !7;
        output.push_str(&format!("sub rsp, {}\n", aligned_size));

        self.stack_size += aligned_size as i32;

        let ty = instances.get(i).unwrap().1.clone();
        match ty {
            Type::int => {
                output.push_str(&format!(
                    "mov dword [rbp - {}], {}\n",
                    off,
                    Self::reg32("rdi")
                ));
            }
            Type::Char | Type::Bool => {
                output.push_str(&format!(
                    "mov byte [rbp - {}], {}\n",
                    off,
                    Self::reg8("rdi")
                ));
            }
            Type::float => {
                output.push_str(&format!("movss [rbp - {}], {}\n", off, "xmm0"));
            }
            _ => {
                output.push_str(&format!("mov qword [rbp - {}], {}\n", off, "rdi"));
            }
        }
        output
    }

    // fn generate_stack_struct_inline(
    //     &mut self,
    //     _name: &str,
    //     instances: Vec<(String, Type)>,
    //     off: usize,
    // ) -> String {
    //     // let struct_layout = layout_fields(&instances);
    //     let mut output = String::new();

    //     output.push_str(&format!("; defining {}\n", _name));
    //     // for fld in &struct_layout.fields {
    //     //     output.push_str(&format!("%define {}.{} {}\n", name, fld.name, fld.offset));
    //     // }

    //     // Compute stack size
    //     let mut stack_offset = 0;
    //     let mut field_offsets = Vec::new();
    //     for instance in &instances {
    //         let size = instance.1.size();
    //         let align = match size {
    //             1 => 1,
    //             4 => 4,
    //             8 => 8,
    //             _ => 8,
    //         };
    //         if stack_offset % align != 0 {
    //             stack_offset += align - (stack_offset % align);
    //         }
    //         field_offsets.push(stack_offset);
    //         stack_offset += size;
    //     }
    //     let aligned_size = (stack_offset + 15) & !15;
    //     output.push_str(&format!("sub rsp, {}\n", aligned_size));

    //     self.stack_size += aligned_size as i32;

    //     // Registers for arguments
    //     let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    //     let save_fp = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4"];
    //     let mut fp_index = 0;

    //     // Initialize fields
    //     for (i, (_, ty)) in instances.iter().enumerate() {
    //         let offset = field_offsets[i] + off;
    //         match ty {
    //             Type::int => {
    //                 output.push_str(&format!(
    //                     "mov dword [rbp - {}], {}\n",
    //                     offset,
    //                     Self::reg32(save_regs[i])
    //                 ));
    //             }
    //             Type::Char | Type::Bool => {
    //                 output.push_str(&format!(
    //                     "mov byte [rbp - {}], {}\n",
    //                     offset,
    //                     Self::reg8(save_regs[i])
    //                 ));
    //             }
    //             Type::float => {
    //                 output.push_str(&format!(
    //                     "movss [rbp - {}], {}\n",
    //                     offset, save_fp[fp_index]
    //                 ));
    //                 fp_index += 1;
    //             }
    //             Type::Struct { name, instances } => {
    //                 // copy struct into parent struct
    //                 // self.generate_stack_struct_inline(name, instances.to_vec(), offset);
    //                 output.push_str("; nested struct\n");
    //             }
    //             _ => {
    //                 output.push_str(&format!("mov qword [rbp - {}], {}\n", offset, save_regs[i]));
    //             }
    //         }
    //     }

    //     output.push_str(&format!("; end {}\n", _name));

    //     output
    // }

    //  fn handle_struct_struct(&self, save_reg: String, struct_: &Type, base_offset: usize) -> String {
    //     let mut output = String::new();

    //     match struct_ {
    //         Type::Struct { name, instances } => {
    //             output.push_str(&format!("; copying struct {} from {}\n", name, save_reg));

    //             let mut src_field_offset = 0;
    //             let mut dest_field_offset = 0;

    //             for (field_name, field_type) in instances {
    //                 let size = field_type.size();
    //                 let align = match size {
    //                     1 => 1,
    //                     2 => 2,
    //                     4 => 4,
    //                     8 => 8,
    //                     _ => 8,
    //                 };

    //                 // Align both source and destination offsets
    //                 if src_field_offset % align != 0 {
    //                     src_field_offset += align - (src_field_offset % align);
    //                 }
    //                 if dest_field_offset % align != 0 {
    //                     dest_field_offset += align - (dest_field_offset % align);
    //                 }

    //                 let src_addr = format!("[{} + {}]", save_reg, src_field_offset);
    //                 let dest_addr = format!("[rbp - {}]", base_offset + dest_field_offset);

    //                 match field_type {
    //                     Type::int => {
    //                         output.push_str(&format!(
    //                             "mov {}, dword {}\n",
    //                             Self::reg32("rax"), src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov dword {}, {}\n",
    //                             dest_addr, Self::reg32("rax")
    //                         ));
    //                     }
    //                     Type::Char | Type::Bool => {
    //                         output.push_str(&format!(
    //                             "mov {}, byte {}\n",
    //                             Self::reg8("rax"), src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov byte {}, {}\n",
    //                             dest_addr, Self::reg8("rax")
    //                         ));
    //                     }
    //                     Type::float => {
    //                         output.push_str(&format!(
    //                             "movss xmm0, dword {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "movss dword {}, xmm0\n", dest_addr
    //                         ));
    //                     }
    //                     Type::Struct { .. } => {
    //                         // For nested structs, get the address of the source nested struct
    //                         output.push_str(&format!(
    //                             "lea rax, {}\n", src_addr
    //                         ));
    //                         output.push_str(&self.handle_struct_struct(
    //                             "rax".to_string(),
    //                             field_type,
    //                             base_offset + dest_field_offset,
    //                         ));
    //                     }
    //                     _ => {
    //                         // For pointers and other 8-byte types
    //                         output.push_str(&format!(
    //                             "mov rax, qword {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov qword {}, rax\n", dest_addr
    //                         ));
    //                     }
    //                 }

    //                 src_field_offset += size;
    //                 dest_field_offset += size;
    //             }
    //         }
    //         _ => {
    //             // Not a struct - this shouldn't happen in this context
    //             output.push_str(&format!("; Warning: {} is not a struct type\n", save_reg));
    //         }
    //     }

    //     output
    // }

    // fn handle_struct_copy_with_layout(&self, save_reg: String, struct_: &Type, base_offset: usize) -> String {
    //     let mut output = String::new();

    //     match struct_ {
    //         Type::Struct { name, instances } => {
    //             output.push_str(&format!("; copying struct {} from {} to [rbp - {}]\n",
    //                 name, save_reg, base_offset));

    //             // Calculate field layout
    //             let field_layout = self._calculate_struct_layout(instances);

    //             for (field_name, field_type, src_offset, dest_offset) in field_layout {
    //                 let src_addr = if src_offset == 0 {
    //                     format!("[{}]", save_reg)
    //                 } else {
    //                     format!("[{} + {}]", save_reg, src_offset)
    //                 };
    //                 let dest_addr = format!("[rbp - {}]", base_offset + dest_offset);

    //                 match field_type {
    //                     Type::int => {
    //                         output.push_str(&format!(
    //                             "mov eax, dword {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov dword {}, eax\n", dest_addr
    //                         ));
    //                     }
    //                     Type::Char | Type::Bool => {
    //                         output.push_str(&format!(
    //                             "mov al, byte {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov byte {}, al\n", dest_addr
    //                         ));
    //                     }
    //                     Type::float => {
    //                         output.push_str(&format!(
    //                             "movss xmm0, dword {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "movss dword {}, xmm0\n", dest_addr
    //                         ));
    //                     }
    //                     Type::Struct { .. } => {
    //                         // For nested structs, recursively copy
    //                         if src_offset == 0 {
    //                             output.push_str(&format!("mov rax, {}\n", save_reg));
    //                         } else {
    //                             output.push_str(&format!("lea rax, [{}+ {}]\n", save_reg, src_offset));
    //                         }
    //                         output.push_str(&self.handle_struct_copy_with_layout(
    //                             "rax".to_string(),
    //                             &field_type,
    //                             base_offset + dest_offset,
    //                         ));
    //                     }
    //                     _ => {
    //                         output.push_str(&format!(
    //                             "mov rax, qword {}\n", src_addr
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov qword {}, rax\n", dest_addr
    //                         ));
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }

    //     output
    // }

    // fn _calculate_struct_layout<'a>(&self, instances: &'a [(String, Type)]) -> Vec<(String, &'a Type, usize, usize)> {
    //     let mut layout = Vec::new();
    //     let mut offset = 0;

    //     for (field_name, field_type) in instances {
    //         let size = field_type.size();
    //         let align = match size {
    //             1 => 1,
    //             2 => 2,
    //             4 => 4,
    //             8 => 8,
    //             _ => 8,
    //         };

    //         // Align offset
    //         if offset % align != 0 {
    //             offset += align - (offset % align);
    //         }

    //         layout.push((field_name.clone(), field_type, offset, offset));
    //         offset += size;
    //     }

    //     layout
    // }

    //     fn generate_stack_struct_inline(
    //     &mut self,
    //     _name: &str,
    //     instances: Vec<(String, Type)>,
    //     base_offset: usize,
    // ) -> (String, usize) { // Return both code and total size
    //     let mut output = String::new();
    //     output.push_str(&format!("; defining {} at offset {}\n", _name, base_offset));

    //     // Compute field layout within this struct
    //     let mut current_offset = 0;
    //     let mut field_info = Vec::new();

    //     for (field_name, field_type) in &instances {
    //         let size = field_type.size();
    //         let align = match size {
    //             1 => 1,
    //             2 => 2,
    //             4 => 4,
    //             8 => 8,
    //             _ => 8,
    //         };

    //         // Align current offset
    //         if current_offset % align != 0 {
    //             current_offset += align - (current_offset % align);
    //         }

    //         field_info.push((field_name.clone(), field_type.clone(), current_offset));
    //         current_offset += size;
    //     }

    //     // Align total struct size
    //     let struct_size = (current_offset + 7) & !7; // 8-byte align structs

    //     // Generate field definitions
    //     for (field_name, _, field_offset) in &field_info {
    //         output.push_str(&format!(
    //             "%define {}.{} {}\n",
    //             _name,
    //             field_name,
    //             field_offset
    //         ));
    //     }

    //     output.push_str(&format!("; {} size: {}\n", _name, struct_size));
    //     (output, struct_size)
    // }

    // fn handle_struct_copy_from_register(
    //     &mut self,
    //     save_reg: &str,
    //     src_struct_type: &Type,
    //     dest_offset: usize,
    // ) -> String {
    //     let mut output = String::new();

    //     match src_struct_type {
    //         Type::Struct { name, instances } => {
    //             output.push_str(&format!(
    //                 "; copying struct {} from {} to [rbp - {}]\n",
    //                 name, save_reg, dest_offset
    //             ));

    //             let mut field_offset = 0;
    //             for (field_name, field_type) in instances {
    //                 let size = field_type.size();
    //                 let align = match size {
    //                     1 => 1,
    //                     2 => 2,
    //                     4 => 4,
    //                     8 => 8,
    //                     _ => 8,
    //                 };

    //                 // Align field offset
    //                 if field_offset % align != 0 {
    //                     field_offset += align - (field_offset % align);
    //                 }

    //                 match field_type {
    //                     Type::int => {
    //                         output.push_str(&format!(
    //                             "mov eax, dword [{} + {}]\n",
    //                             save_reg, field_offset
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov dword [rbp - {}], eax\n",
    //                             dest_offset + field_offset
    //                         ));
    //                     }
    //                     Type::Char | Type::Bool => {
    //                         output.push_str(&format!(
    //                             "mov al, byte [{} + {}]\n",
    //                             save_reg, field_offset
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov byte [rbp - {}], al\n",
    //                             dest_offset + field_offset
    //                         ));
    //                     }
    //                     Type::float => {
    //                         output.push_str(&format!(
    //                             "movss xmm0, dword [{} + {}]\n",
    //                             save_reg, field_offset
    //                         ));
    //                         output.push_str(&format!(
    //                             "movss dword [rbp - {}], xmm0\n",
    //                             dest_offset + field_offset
    //                         ));
    //                     }
    //                     Type::Struct { name: nested_name, instances: nested_instances } => {
    //                         // For nested structs, recursively copy
    //                         output.push_str(&format!(
    //                             "lea rax, [{} + {}]\n",
    //                             save_reg, field_offset
    //                         ));
    //                         output.push_str(&self.handle_struct_copy_from_register(
    //                             "rax",
    //                             field_type,
    //                             dest_offset + field_offset,
    //                         ));
    //                     }
    //                     _ => {
    //                         output.push_str(&format!(
    //                             "mov rax, qword [{} + {}]\n",
    //                             save_reg, field_offset
    //                         ));
    //                         output.push_str(&format!(
    //                             "mov qword [rbp - {}], rax\n",
    //                             dest_offset + field_offset
    //                         ));
    //                     }
    //                 }

    //                 field_offset += size;
    //             }
    //         }
    //         _ => {
    //             // Not a struct, shouldn't happen
    //         }
    //     }

    //     output
    // }

    // fn generate_stack_struct_with_args(
    //     &mut self,
    //     _name: &str,
    //     instances: Vec<(String, Type)>,
    //     base_offset: usize,
    // ) -> String {
    //     let mut output = String::new();

    //     // First pass: calculate layout and total size
    //     let mut current_offset = 0;
    //     let mut field_info = Vec::new();

    //     for (field_name, field_type) in &instances {
    //         let size = field_type.size();
    //         let align = match size {
    //             1 => 1,
    //             2 => 2,
    //             4 => 4,
    //             8 => 8,
    //             _ => 8,
    //         };

    //         // Align current offset
    //         if current_offset % align != 0 {
    //             current_offset += align - (current_offset % align);
    //         }

    //         field_info.push((field_name.clone(), field_type.clone(), current_offset));
    //         current_offset += size;
    //     }

    //     let struct_size = (current_offset + 15) & !15; // 16-byte align for stack

    //     // Allocate stack space
    //     output.push_str(&format!("sub rsp, {}\n", struct_size));
    //     self.stack_size += struct_size as i32;

    //     // Generate field offset defines
    //     for (field_name, _, field_offset) in &field_info {
    //         output.push_str(&format!(
    //             "%define {}.{} {}\n",
    //             _name,
    //             field_name,
    //             field_offset
    //         ));
    //     }

    //     // Argument registers
    //     let int_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    //     let float_regs = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5"];

    //     let mut int_reg_idx = 0;
    //     let mut float_reg_idx = 0;

    //     // Initialize fields from arguments
    //     for (field_name, field_type, field_offset) in field_info {
    //         let dest_offset = base_offset + field_offset;

    //         match field_type {
    //             Type::int => {
    //                 if int_reg_idx < int_regs.len() {
    //                     output.push_str(&format!(
    //                         "mov dword [rbp - {}], {}\n",
    //                         dest_offset,
    //                         Self::reg32(int_regs[int_reg_idx])
    //                     ));
    //                     int_reg_idx += 1;
    //                 }
    //             }
    //             Type::Char | Type::Bool => {
    //                 if int_reg_idx < int_regs.len() {
    //                     output.push_str(&format!(
    //                         "mov byte [rbp - {}], {}\n",
    //                         dest_offset,
    //                         Self::reg8(int_regs[int_reg_idx])
    //                     ));
    //                     int_reg_idx += 1;
    //                 }
    //             }
    //             Type::float => {
    //                 if float_reg_idx < float_regs.len() {
    //                     output.push_str(&format!(
    //                         "movss dword [rbp - {}], {}\n",
    //                         dest_offset,
    //                         float_regs[float_reg_idx]
    //                     ));
    //                     float_reg_idx += 1;
    //                 }
    //             }
    //             Type::Struct { .. } => {
    //                 // For struct arguments, the struct is passed by pointer
    //                 if int_reg_idx < int_regs.len() {
    //                     output.push_str(&self.handle_struct_copy_from_register(
    //                         int_regs[int_reg_idx],
    //                         &field_type,
    //                         dest_offset,
    //                     ));
    //                     int_reg_idx += 1;
    //                 }
    //             }
    //             _ => {
    //                 if int_reg_idx < int_regs.len() {
    //                     output.push_str(&format!(
    //                         "mov qword [rbp - {}], {}\n",
    //                         dest_offset,
    //                         int_regs[int_reg_idx]
    //                     ));
    //                     int_reg_idx += 1;
    //                 }
    //             }
    //         }
    //     }

    //     output.push_str(&format!("; end {} initialization\n", _name));
    //     output
    // }

    // // Helper function to access nested struct fields
    // fn generate_struct_field_access(
    //     struct_base_reg: &str,
    //     struct_type: &Type,
    //     field_path: &[String],
    // ) -> String {
    //     if field_path.is_empty() {
    //         return struct_base_reg.to_string();
    //     }

    //     match struct_type {
    //         Type::Struct { name, instances } => {
    //             let field_name = &field_path[0];
    //             let mut field_offset = 0;

    //             for (fname, ftype) in instances {
    //                 let size = ftype.size();
    //                 let align = match size {
    //                     1 => 1,
    //                     2 => 2,
    //                     4 => 4,
    //                     8 => 8,
    //                     _ => 8,
    //                 };

    //                 // Align field offset
    //                 if field_offset % align != 0 {
    //                     field_offset += align - (field_offset % align);
    //                 }

    //                 if fname == field_name {
    //                     if field_path.len() == 1 {
    //                         // Final field
    //                         return format!("[{} + {}]", struct_base_reg, field_offset);
    //                     } else {
    //                         // Nested struct access
    //                         let remaining_path = &field_path[1..];
    //                         // Load address of nested struct
    //                         return format!("lea rax, [{} + {}]\n{}",
    //                             struct_base_reg,
    //                             field_offset,
    //                             Self::generate_struct_field_access("rax", ftype, remaining_path)
    //                         );
    //                     }
    //                 }

    //                 field_offset += size;
    //             }
    //         }
    //         _ => {}
    //     }

    //     "".to_string()
    // }

    fn generate_stack_struct_inline(
        &mut self,
        name: &str,
        instances: Vec<(String, Type)>,
        _off: usize,
        ofst: Option<usize>,
    ) -> String {
        let mut output = String::new();
        output.push_str(&format!("; defining {}\n", name));

        // Compute field offsets with alignment
        let mut stack_offset = 0;
        let mut field_offsets = Vec::new();
        for (_, ty) in &instances {
            let mut size = ty.size();

            if let Type::Struct { .. } = ty {
                size = 8;
            }

            stack_offset += size;
            field_offsets.push(stack_offset);
            stack_offset += size;
        }
        let aligned_size = (stack_offset + 15) & !15;
        output.push_str(&format!("sub rsp, {}\n", aligned_size));
        self.stack_size += aligned_size as i32;

        // Registers for arguments
        let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let save_fp = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4"];
        let mut gp_index = ofst.unwrap_or(0);
        let mut fp_index = 0;

        let mut offset = 0;

        // Initialize fields
        for (i, (_fname, ty)) in instances.iter().enumerate() {
            match ty {
                Type::int => {
                    output.push_str(&format!(
                        "mov dword [rsp + {}], {}\n",
                        offset,
                        Self::reg32(save_regs[gp_index])
                    ));
                    gp_index += 1;
                }
                Type::Char | Type::Bool => {
                    output.push_str(&format!(
                        "mov byte [rsp + {}], {}\n",
                        offset,
                        Self::reg8(save_regs[gp_index])
                    ));
                    gp_index += 1;
                }
                Type::float => {
                    output.push_str(&format!(
                        "movss [rsp + {}], {}\n",
                        offset, save_fp[fp_index]
                    ));
                    fp_index += 1;
                }
                _ => {
                    output.push_str(&format!(
                        "mov qword [rsp + {}], {}\n",
                        offset, save_regs[gp_index]
                    ));
                    gp_index += 1;
                }
            }

            offset += field_offsets[i];
        }

        output.push_str(&format!("; end {}\n", name));
        output
    }

    fn generate_struct(&mut self, name: &str, instances: Vec<(String, Type)>, union: bool) {
        let struct_layout = layout_fields(&instances);

        self.structures.insert(
            name.to_string(),
            (
                struct_layout,
                Stmt::StructDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    union,
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

    fn parse_escapes(s: &str) -> Vec<char> {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    let escaped = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        '0' => '\0',
                        _ => next,
                    };
                    result.push(escaped);
                    chars.next();
                } else {
                    result.push('\\');
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn handle_expr(&mut self, expr: &Expr, _ident: Option<String>) -> Option<String> {
        match expr {
            Expr::Cast { expr, target_type } => {
                let src = self.handle_expr(expr, None)?;
                match target_type {
                    Type::int => {
                        if src.starts_with("xmm") {
                            let dst = self.regs.pop_front().expect("No gp reg");
                            self.output.push_str(&format!(
                                "cvttss2si {}, {}\n",
                                Self::reg32(&dst),
                                src
                            ));
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
                                .push_str(&format!("cvtsi2ss {dst}, {}\n", Self::reg32(&src)));
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
                            Self::reg32(&val_reg)
                        ));
                    }
                    Type::Bool | Type::Char => {
                        self.output.push_str(&format!(
                            "mov byte [{reg} + {field_offset}], {}\n",
                            Self::reg8(&val_reg)
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

                    // println!("{args:?}");

                    let size = args[0].get_type().size();

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
                //     .push_str(&format!("mov {}, '{n}'\n", Self::reg8(&av_reg)));
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
                            Self::reg32(&val_reg)
                        ));
                    }
                    Type::Char | Type::Bool => {
                        self.output.push_str(&format!(
                            "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                            Self::reg8(&val_reg)
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
                                .push_str(&format!("mov {}, [{name}]\n", Self::reg32(&av_reg)));
                            return Some(av_reg);
                        }
                        Type::Char | Type::Bool => {
                            self.output
                                .push_str(&format!("mov {}, [{name}]\n", Self::reg8(&av_reg)));
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
                        self.output.push_str(&format!(
                            "mov {}, dword [rbp - {off}]\n",
                            Self::reg32(&av_reg)
                        ));
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
                            map.insert(fname.as_str(), fty);
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
                                self.output.push_str(&format!(
                                    "cvttss2si {}, {}\n",
                                    Self::reg32(&gp),
                                    r
                                ));
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

                        self.output
                            .push_str(&format!("mov qword [rbp - {offset}], [rbp - {var_off}]"));
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
                                    Self::reg32(&val_reg)
                                ));
                            }
                            Type::Char | Type::Bool => {
                                self.output.push_str(&format!(
                                    "mov byte [{struct_ptr_reg} + {field_offset}], {}\n",
                                    Self::reg8(&val_reg)
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
                                .push_str(&format!("mov {}, byte [{ptr}]\n", Self::reg8(&reg)));
                        }
                        Type::int => {
                            self.output
                                .push_str(&format!("mov {}, dword [{ptr}]\n", Self::reg32(&reg)));
                        }
                        Type::Pointer(inside) => match **inside {
                            Type::Bool | Type::Char => {
                                self.output
                                    .push_str(&format!("mov {}, byte [{ptr}]\n", Self::reg8(&reg)));
                            }
                            Type::int => {
                                self.output.push_str(&format!(
                                    "mov {}, dword [{ptr}]\n",
                                    Self::reg32(&reg)
                                ));
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
                                Self::reg32(&val_reg)
                            ));
                        }
                        Type::Char | Type::Bool => {
                            self.output.push_str(&format!(
                                "mov {}, byte [{ptr_reg} + {field_offset}]\n",
                                Self::reg8(&val_reg)
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
                                Self::reg8(&val_reg)
                            ));
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
                                    Self::reg32(&val_reg)
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
                self.output
                    .push_str(&format!("mov {}, qword [rsp]\n", lhs));
                self.output.push_str("add rsp, 8\n");
                self.stack_size -= 8;

                match op {
                    BinaryOp::Add => {
                        let lhs32 = Self::reg32(&lhs);
                        let rhs32 = Self::reg32(&rhs);
                        self.output.push_str(&format!("add {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Sub => {
                        let lhs32 = Self::reg32(&lhs);
                        let rhs32 = Self::reg32(&rhs);
                        self.output.push_str(&format!("sub {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Mul => {
                        let lhs32 = Self::reg32(&lhs);
                        let rhs32 = Self::reg32(&rhs);
                        self.output.push_str(&format!("imul {lhs32}, {rhs32}\n"));
                        self.regs.push_back(rhs);
                        Some(lhs)
                    }
                    BinaryOp::Div => {
                        // Signed 32-bit division: edx:eax / r/m32 -> eax
                        let lhs32 = Self::reg32(&lhs);
                        let rhs32 = Self::reg32(&rhs);
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
