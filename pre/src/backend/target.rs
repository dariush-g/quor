use crate::{
    backend::{CodegenCtx, FrameLayout},
    ir::block::{GlobalDef, IRFunction, IRInstruction, Terminator},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Target {
    X86,
    ARM,
}

pub trait TargetEmitter: std::fmt::Debug {
    fn add_global_const(&mut self, constant: GlobalDef) {}

    fn prologue(&mut self, frame: &FrameLayout, func: &IRFunction);
    fn epilogue(&mut self, frame: &FrameLayout, func: &IRFunction);

    fn emit_inst(&mut self, inst: &IRInstruction, frame: &FrameLayout, ctx: &mut CodegenCtx);
    fn emit_term(&mut self, term: &Terminator, frame: &FrameLayout, ctx: &mut CodegenCtx);
}
