use std::{collections::HashMap, fmt::format};

use crate::{
    backend::{
        lir::{
            aarch64::{A64RegFpr, A64RegGpr},
            regalloc::{LFunction, LInst, LTerm},
        },
        target::TargetEmitter,
        *,
    },
    mir::block::*,
};

#[derive(Debug, Default)]
pub struct ARMEmitter {
    target_regs: A64Regs,
}

impl TargetEmitter for ARMEmitter {
    type Reg = A64RegGpr;
    type FpReg = A64RegFpr;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        let ty = match constant.value {
            GlobalValue::String(s) => &format!(".asciz \"{s}\""),
            GlobalValue::Int(i) => &format!(".word {i}"),
            GlobalValue::Float(f) => &format!(".float {f}"),
            GlobalValue::Bool(b) => &format!(".byte {b}"),
            GlobalValue::Zeroed(_) => ".byte 0",
            GlobalValue::Char(c) => &format!(".byte {c}"),
            GlobalValue::Struct(expr) => todo!(), // TODO
            _ => "",
        };

        let global = format!("__q_g_{}:\n {}", constant.id, ty);

        global
    }

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        match inst {
            LInst::Add { dst, a, b } => {
                todo!()
            }
            LInst::Sub { dst, a, b } => todo!(),
            LInst::Mul { dst, a, b } => todo!(),
            LInst::Div { dst, a, b } => todo!(),
            LInst::Mod { dst, a, b } => todo!(),
            LInst::CmpSet { dst, op, a, b } => todo!(),
            LInst::Cast { dst, src, ty } => todo!(),
            LInst::Load { dst, addr, ty } => todo!(),
            LInst::Store { src, addr, ty } => todo!(),
            LInst::Call { dst, func, args } => todo!(),
            LInst::Mov { dst, src } => todo!(),
            LInst::Lea { dst, addr } => todo!(),
        }
    }

    fn t_emit_term(
        &mut self,
        term: &LTerm<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn t_operand(&self, operand: &lir::regalloc::Operand<Self::Reg, Self::FpReg>) -> String {
        todo!()
    }
    fn t_loc(&self, loc: lir::regalloc::Loc<Self::Reg, Self::FpReg>) -> String {
        todo!()
    }

    fn t_addr(&self, loc: lir::regalloc::Addr<Self::Reg>) -> String {
        todo!()
    }

    fn t_prologue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn t_epilogue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn generate_ctx(func: &LFunction<Self::Reg, Self::FpReg>) -> CodegenCtx<Self::Reg, Self::FpReg> {
        todo!()
    }
}
