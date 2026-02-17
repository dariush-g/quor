use crate::{backend::{lir::x86_64::{X86RegFpr, X86RegGpr}, target::TargetEmitter}, mir::block::*};

#[derive(Debug, Default)]
pub struct X86Emitter {}

impl TargetEmitter for X86Emitter {
    type Reg = X86RegGpr;
    type FpReg = X86RegFpr;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        todo!()
    }

    fn t_prologue(
        &mut self,
        frame: &crate::backend::FrameLayout,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn t_epilogue(
        &mut self,
        frame: &crate::backend::FrameLayout,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn t_emit_inst(
        &mut self,
        inst: &crate::backend::lir::regalloc::LInst<Self::Reg, Self::FpReg>,
        frame: &crate::backend::FrameLayout,
        ctx: &mut crate::backend::CodegenCtx,
    ) -> String {
        todo!()
    }

    fn t_emit_term(
        &mut self,
        term: &crate::backend::lir::regalloc::LTerm<Self::Reg, Self::FpReg>,
        frame: &crate::backend::FrameLayout,
        ctx: &mut crate::backend::CodegenCtx,
    ) -> String {
        todo!()
    }

    fn generate_stack_frame(
        &mut self,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> crate::backend::FrameLayout {
        todo!()
    }

    fn generate_function(
        &mut self,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }
    
    fn t_loc(&self, loc: crate::backend::lir::regalloc::Loc<Self::Reg, Self::FpReg>) -> String {
        todo!()
    }
    
    fn t_addr(&self, loc: crate::backend::lir::regalloc::Addr<Self::Reg>) -> String {
        todo!()
    }
}
