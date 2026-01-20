use crate::{backend::target::TargetEmitter, ir::block::*};

#[derive(Debug, Default)]
pub struct ARMEmitter {}

impl TargetEmitter for ARMEmitter {
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

pub fn aarch64_add(reg: Value, left: Value, right: Value) -> Value {
    reg
}
