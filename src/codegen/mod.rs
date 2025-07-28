mod print;

use crate::{
    codegen::print::{add_print, print},
    lexer::ast::Stmt,
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
            match stmt {
                Stmt::VarDecl {
                    name,
                    var_type,
                    value,
                } => {
                    match var_type {
                        crate::lexer::ast::Type::int => {
                            match value {
                                // crate::lexer::ast::Expr::IntLiteral(_) => todo!(),
                                // crate::lexer::ast::Expr::FloatLiteral(_) => todo!(),
                                // crate::lexer::ast::Expr::BoolLiteral(_) => todo!(),
                                // crate::lexer::ast::Expr::CharLiteral(_) => todo!(),
                                // crate::lexer::ast::Expr::Variable(_) => todo!(),
                                // crate::lexer::ast::Expr::Assign { name, value } => todo!(),
                                crate::lexer::ast::Expr::Binary {
                                    left,
                                    op,
                                    right,
                                    result_type,
                                } => {
                                    let left_ = match **left {
                                        crate::lexer::ast::Expr::IntLiteral(n) => n,
                                        _ => 0,
                                    };

                                    let right_ = match **right {
                                        crate::lexer::ast::Expr::IntLiteral(n) => n,
                                        _ => 0,
                                    };

                                    match op {
                                    crate::lexer::ast::BinaryOp::Add => code
                                        .output
                                        .push_str(&format!("mov eax, {left_}\nadd eax, {right_}\nmov ebx, eax\n")),
                                    _ => {}
                                }
                                }
                                // crate::lexer::ast::Expr::Unary { op, expr, result_type } => todo!(),
                                // crate::lexer::ast::Expr::Call { name, args, return_type } => todo!(),
                                // crate::lexer::ast::Expr::Cast { expr, target_type } => todo!(),
                                // crate::lexer::ast::Expr::Array(exprs, _) => todo!(),
                                // crate::lexer::ast::Expr::ArrayAccess { array, index, element_type } => todo!(),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                // Stmt::FunDecl { name, params, return_type, body } => todo!(),
                // Stmt::If { condition, then_stmt, else_stmt } => todo!(),
                // Stmt::While { condition, body } => todo!(),
                // Stmt::For { init, condition, update, body } => todo!(),
                // Stmt::Block(stmts) => todo!(),
                Stmt::Expression(expr) => match expr {
                    // crate::lexer::ast::Expr::IntLiteral(_) => todo!(),
                    // crate::lexer::ast::Expr::FloatLiteral(_) => todo!(),
                    // crate::lexer::ast::Expr::BoolLiteral(_) => todo!(),
                    // crate::lexer::ast::Expr::CharLiteral(_) => todo!(),
                    // crate::lexer::ast::Expr::Variable(_) => todo!(),
                    // crate::lexer::ast::Expr::Assign { name, value } => todo!(),
                    // crate::lexer::ast::Expr::Binary {
                    //     left,
                    //     op,
                    //     right,
                    //     result_type,
                    // } => todo!(),
                    // crate::lexer::ast::Expr::Unary {
                    //     op,
                    //     expr,
                    //     result_type,
                    // } => todo!(),
                    crate::lexer::ast::Expr::Call {
                        name,
                        args,
                        return_type,
                    } => {} // crate::lexer::ast::Expr::Cast { expr, target_type } => todo!(),
                    // crate::lexer::ast::Expr::Array(exprs, _) => todo!(),
                    // crate::lexer::ast::Expr::ArrayAccess {
                    //     array,
                    //     index,
                    //     element_type,
                    // } => todo!(),
                    _ => {}
                },
                // Stmt::Return(expr) => todo!(),
                // Stmt::Break => todo!(),
                // Stmt::Continue => todo!(),
                _ => {}
            }
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
}
