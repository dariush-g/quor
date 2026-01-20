use crate::{backend::target::TargetEmitter, ir::block::*};

#[derive(Debug, Default)]
pub struct X86Emitter {}

impl TargetEmitter for X86Emitter {
    fn prologue(&mut self, frame: &super::FrameLayout, func: &IRFunction) {
        todo!()
    }

    fn epilogue(&mut self, frame: &super::FrameLayout, func: &IRFunction) {
        todo!()
    }

    fn emit_inst(
        &mut self,
        inst: &IRInstruction,
        frame: &super::FrameLayout,
        ctx: &mut super::CodegenCtx,
    ) {
        todo!()
    }

    fn emit_term(
        &mut self,
        term: &Terminator,
        frame: &super::FrameLayout,
        ctx: &mut super::CodegenCtx,
    ) {
        todo!()
    }
}
