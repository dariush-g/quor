use crate::{
    backend::{
        CodegenCtx, FrameLayout,
        lir::allocation::{LFunction, LInst, LTerm},
    },
    mir::block::{GlobalDef, IRFunction, IRInstruction, Terminator},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Target {
    X86,
    ARM,
}

pub trait TargetEmitter: std::fmt::Debug {
    type Reg: Copy + Eq + std::hash::Hash + std::fmt::Debug;
    type FpReg: Copy + Eq + std::hash::Hash + std::fmt::Debug;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String;

    fn t_prologue(
        &mut self,
        frame: &FrameLayout,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String;
    fn t_epilogue(
        &mut self,
        frame: &FrameLayout,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String;

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
    ) -> String;

    fn t_emit_term(
        &mut self,
        term: &LTerm<Self::Reg, Self::FpReg>,
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
    ) -> String;

    fn generate_stack_frame(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> FrameLayout;
    fn generate_function(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> String;
}
