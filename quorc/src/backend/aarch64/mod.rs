use std::{collections::HashMap, fmt::format};

use crate::{
    backend::{CodegenCtx, target::TargetEmitter},
    mir::block::*,
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
        #[cfg(target_os = "macos")]
        let function_name = format!("_{}", func.name.clone());
        #[cfg(not(target_os = "macos"))]
        let function_name = func.name.clone();
        format!(
            "{function_name}:\nstp x29, x30, [sp, #-16]!\nmov x29, sp\nsub sp, sp, #{}",
            frame.frame_size
        )
    }

    fn t_epilogue(&mut self, frame: &super::FrameLayout, func: &IRFunction) -> String {
        format!(
            "add sp, sp, #{}\nldp x29, x30, [sp], #16\nret\n",
            frame.frame_size
        )
    }

    fn t_emit_inst(
        &mut self,
        inst: &IRInstruction,
        frame: &super::FrameLayout,
        ctx: &mut super::CodegenCtx,
    ) -> String {
        match inst {
            IRInstruction::Add { reg, left, right } => todo!(),
            IRInstruction::Sub { reg, left, right } => todo!(),
            IRInstruction::Mul { reg, left, right } => todo!(),
            IRInstruction::Div { reg, left, right } => todo!(),
            IRInstruction::Mod { reg, left, right } => todo!(),
            IRInstruction::Eq { reg, left, right } => todo!(),
            IRInstruction::Ne { reg, left, right } => todo!(),
            IRInstruction::Lt { reg, left, right } => todo!(),
            IRInstruction::Le { reg, left, right } => todo!(),
            IRInstruction::Ge { reg, left, right } => todo!(),
            IRInstruction::Gt { reg, left, right } => todo!(),
            IRInstruction::Cast { reg, src, ty } => todo!(),
            IRInstruction::Load {
                reg,
                addr,
                offset,
                ty,
            } => todo!(),
            IRInstruction::Store {
                value,
                addr,
                offset,
                ty,
            } => todo!(),
            IRInstruction::Gep {
                dest,
                base,
                index,
                scale,
            } => todo!(),
            IRInstruction::Call { reg, func, args } => {
                let call = format!("BL {func}\n");
                if let Some(dest_reg) = reg {}
                call
            }
            IRInstruction::Move { dest, from } => todo!(),
            IRInstruction::AddressOf { dest, src } => todo!(),
            IRInstruction::Declaration(at_decl) => todo!(),
        }
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

        if off % 16 != 0 {
            off += 16 - (off % 16);
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
        println!("{frame:?}");
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
