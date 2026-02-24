use crate::{
    backend::{
        CodegenCtx,
        lir::regalloc::{Addr, LFunction, LInst, LTerm, Loc, Operand},
    },
    midend::mir::block::GlobalDef,
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

    fn t_loc(&self, loc: Loc<Self::Reg, Self::FpReg>) -> String;

    fn t_addr(&self, loc: Addr<Self::Reg>) -> String;

    fn t_prologue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String;

    fn t_epilogue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String;

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String;

    fn t_emit_term(
        &mut self,
        term: &LTerm<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String;

    fn t_operand(&self, operand: &Operand<Self::Reg, Self::FpReg>) -> String;

    fn generate_function(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> String {
        let mut func_asm = String::new();
        let ctx = &mut Self::generate_ctx(func);
        func_asm.push_str(&self.t_prologue(ctx, func));

        for block in func.blocks.clone() {
            for inst in block.insts {
                func_asm.push_str(&self.t_emit_inst(&inst, ctx));
            }
            func_asm.push_str(&self.t_emit_term(&block.term, ctx));
        }

        func_asm.push_str(&self.t_epilogue(ctx, func));

        func_asm
    }

    fn t_extern(&self, ext: String) -> String;

    fn generate_ctx(
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> CodegenCtx<'_, Self::Reg, Self::FpReg>;
}
