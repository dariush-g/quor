use std::{collections::HashMap, fmt::format};

use crate::{
    backend::{CodegenCtx, target::TargetEmitter},
    ir::block::*,
};

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
        let mut off = 0_i32;
        let mut local_off = HashMap::new();
        for block in func.blocks.clone() {
            for instr in block.instructions {
                if let IRInstruction::Store { addr, ty, .. } = instr
                    && let Value::Local(id) = addr
                    && !local_off.contains_key(&id)
                {
                    local_off.insert(id, off);
                    off += ty.size() as i32;
                };
            }
        }
        
        super::FrameLayout {
            local_off,
            vreg_off: HashMap::new(),
            frame_size: off,
            align: 16,
        }
    }

    fn generate_function(&mut self, func: &IRFunction) -> String {
        let mut assembly_function = String::new();
        let frame = self.generate_stack_frame(func);
        assembly_function.push_str(&self.t_prologue(&frame, func));

        let block_labels: HashMap<BlockId, &IRBlock> =
            func.blocks.iter().map(|block| (block.id, block)).collect();

        let context = CodegenCtx {
            func,
            frame: &frame,
            block_labels,
            current_block: func.entry,
            next_tmp_label: 0, // TODO
        };

        assembly_function.push_str(&self.t_epilogue(&frame, func));
        assembly_function
    }
}

pub fn aarch64_add(reg: Value, left: Value, right: Value) -> String {
    String::new()
}
