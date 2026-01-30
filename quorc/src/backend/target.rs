use crate::{
    backend::{CodegenCtx, FrameLayout},
    mir::block::{GlobalDef, IRFunction, IRInstruction, Terminator},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Target {
    X86,
    ARM,
}

pub trait TargetEmitter: std::fmt::Debug {
    fn t_add_global_const(&mut self, constant: GlobalDef) -> String;

    fn t_prologue(&mut self, frame: &FrameLayout, func: &IRFunction) -> String;
    fn t_epilogue(&mut self, frame: &FrameLayout, func: &IRFunction) -> String;

    fn t_emit_inst(
        &mut self,
        inst: &IRInstruction,
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
    ) -> String;
    
    fn t_emit_term(
        &mut self,
        term: &Terminator,
        frame: &FrameLayout,
        ctx: &mut CodegenCtx,
    ) -> String;

    fn generate_stack_frame(&mut self, func: &IRFunction) -> FrameLayout;
    fn generate_function(&mut self, func: &IRFunction) -> String;
}
