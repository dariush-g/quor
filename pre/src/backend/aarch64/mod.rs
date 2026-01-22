use std::fmt::format;

use crate::{backend::target::TargetEmitter, ir::block::*};

#[derive(Debug, Default)]
pub struct ARMEmitter {}

impl TargetEmitter for ARMEmitter {
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

    fn t_prologue(&mut self, frame: &super::FrameLayout, func: &IRFunction) -> String {
        todo!()
    }

    fn t_epilogue(&mut self, frame: &super::FrameLayout, func: &IRFunction) -> String {
        todo!()
    }

    fn t_emit_inst(
        &mut self,
        inst: &IRInstruction,
        frame: &super::FrameLayout,
        ctx: &mut super::CodegenCtx,
    ) -> String {
        todo!()
    }

    fn t_emit_term(
        &mut self,
        term: &Terminator,
        frame: &super::FrameLayout,
        ctx: &mut super::CodegenCtx,
    ) -> String {
        todo!()
    }
    
    fn generate_stack_frame(&mut self, func: &IRFunction) -> super::FrameLayout {
        todo!()
    }
    
    fn generate_function(&mut self, func: &IRFunction) -> String {
        todo!()
    }
}

pub fn aarch64_add(reg: Value, left: Value, right: Value) -> Value {
    reg
}
