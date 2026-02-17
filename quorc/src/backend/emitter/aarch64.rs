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
pub struct ARMEmitter {}

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

    fn t_prologue(
        &mut self,
        frame: &FrameLayout,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        #[cfg(target_os = "macos")]
        let function_name = format!("_{}", func.name.clone());
        #[cfg(not(target_os = "macos"))]
        let function_name = func.name.clone();
        format!(
            "{function_name}:\nstp x29, x30, [sp, #-16]!\nmov x29, sp\nsub sp, sp, #{}",
            frame.frame_size
        )
    }

    fn t_epilogue(
        &mut self,
        frame: &FrameLayout,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        format!(
            "add sp, sp, #{}\nldp x29, x30, [sp], #16\nret\n",
            frame.frame_size
        )
    }

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
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
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
    ) -> String {
        todo!()
    }

    fn generate_stack_frame(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> FrameLayout {
        todo!()
    }

    fn generate_function(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> String {
        let mut assembly_function = String::new();
        let frame = self.generate_stack_frame(func);
        println!("{frame:?}");
        assembly_function.push_str(&self.t_prologue(&frame, func));

        assembly_function.push_str(&self.t_epilogue(&frame, func));
        assembly_function
    }

    fn t_loc(&self, loc: lir::regalloc::Loc<Self::Reg, Self::FpReg>) -> String {
        todo!()
    }

    fn t_addr(&self, loc: lir::regalloc::Addr<Self::Reg>) -> String {
        todo!()
    }
}
