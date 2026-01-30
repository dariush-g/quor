use crate::{backend::target::TargetEmitter, mir::block::*};

#[derive(Debug, Default)]
pub struct X86Emitter {}

impl TargetEmitter for X86Emitter {
    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        todo!()
    }

    fn t_prologue(&mut self, frame: &crate::backend::FrameLayout, func: &IRFunction) -> String {
        todo!()
    }

    fn t_epilogue(&mut self, frame: &crate::backend::FrameLayout, func: &IRFunction) -> String {
        todo!()
    }

    fn t_emit_inst(
        &mut self,
        inst: &IRInstruction,
        frame: &crate::backend::FrameLayout,
        ctx: &mut crate::backend::CodegenCtx,
    ) -> String {
        todo!()
    }

    fn t_emit_term(
        &mut self,
        term: &Terminator,
        frame: &crate::backend::FrameLayout,
        ctx: &mut crate::backend::CodegenCtx,
    ) -> String {
        todo!()
    }

    fn generate_stack_frame(&mut self, func: &IRFunction) -> crate::backend::FrameLayout {
        todo!()
    }

    fn generate_function(&mut self, func: &IRFunction) -> String {
        todo!()
    }
}
