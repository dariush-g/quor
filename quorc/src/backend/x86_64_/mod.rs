use crate::{
    backend::x86_64_::r#struct::fields::{StructLayout, layout_of_struct},
    frontend::ast::{BinaryOp, Expr, Stmt, Type, UnaryOp},
};

mod expr;
mod func;
mod regs;
mod stmt;
mod r#struct;
mod union;

//
// add codegen for $param in inline assembly so the assembly can use local vars.
// > get var name from stack
//

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
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
pub fn align_up(x: usize, a: usize) -> usize {
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

pub fn size_align_of(ty: &Type) -> (usize, usize) {
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
    #[allow(unused_assignments)]
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
            if let Stmt::AtDecl(decl, param, val, _) = stmt
                && (decl.as_str() == "define" || decl.as_str() == "defines")
            {
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
            if let Stmt::FunDecl {
                name,
                params,
                body,
                attributes,
                ..
            } = stmt
            {
                if name == "main" {
                    has_main = true;
                    if !params.is_empty() {
                        if params[0].1 == Type::int
                            && let Type::Pointer(boxed_ty) = &params[1].1
                            && let Type::Pointer(inside) = *boxed_ty.clone()
                            && *inside == Type::Char
                        {
                            code.generate_function("main", params.clone(), body, attributes);
                        } else {
                            panic!("unexpected parameters for main function");
                        }
                    } else {
                        code.generate_function("main", vec![], body, attributes);
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

        let mut print = fs::read_to_string(format!("{manifest_dir}/lib/x86_64/io.asm"))
            .unwrap_or_else(|_| panic!("Error importing io"));

        print.push('\n');

        code.output.push_str(&print);

        let mut mem = fs::read_to_string(format!("{manifest_dir}/lib/x86_64/mem.asm"))
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
        self.output.push_str("xor rax, rax\n");
        self.output.push_str(&format!("call {target}\n"));
        if need_pad {
            self.output.push_str("add rsp, 8\n");
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
}
