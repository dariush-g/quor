mod print;

use crate::{
    codegen::print::{add_print, print},
    lexer::ast::{Expr, Stmt},
};

pub struct CodeGen {
    output: String,
    jmp_count: u32,
}

impl CodeGen {
    pub fn generate(stmts: &Vec<Stmt>) -> String {
        // var x: int = 1 + 1;

        let mut code = CodeGen {
            output: String::new(),
            jmp_count: 0,
        };

        code.output.push_str(&add_print());

        for stmt in stmts {
            code.handle_stmt(stmt);
        }

        //     mov rax, 60
        //     xor rdi, rdi
        //     syscall

        code.output.push_str(&format!("global _start\n_start:\n"));

        // code.output.push_str(&print("hello".to_owned()));

        code.output
            .push_str(&format!("mov rax, 60\nxor rdi, rdi\nsyscall"));

        code.output
    }

    fn handle_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name,
                var_type,
                value,
            } => todo!(),
            Stmt::FunDecl {
                name,
                params,
                return_type,
                body,
            } => todo!(),
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => todo!(),
            Stmt::While { condition, body } => todo!(),
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => todo!(),
            Stmt::Block(stmts) => todo!(),
            Stmt::Expression(expr) => todo!(),
            Stmt::Return(expr) => todo!(),
            Stmt::Break => todo!(),
            Stmt::Continue => todo!(),
        }
    }

    fn handle_expr(&mut self, expr: &Expr) {}
}
