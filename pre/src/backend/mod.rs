use std::collections::HashMap;

use crate::{
    backend::{
        aarch64::ARMEmitter,
        target::{Target, TargetEmitter},
        x86_64::X86Emitter,
    },
    ir::block::*,
};

pub mod aarch64;
pub mod target;
pub mod x86_64;
pub mod x86_64_;

#[derive(Debug)]
pub struct Codegen {
    pub target: Target,
    pub emitter: Box<dyn TargetEmitter>,
    pub asm: AsmEmitter,
}

impl Codegen {
    pub fn generate(ir_program: IRProgram) -> String {
        #[cfg(target_arch = "aarch64")]
        let target = Target::ARM;
        #[cfg(target_arch = "x86_64")]
        let target = Target::X86;

        let codegen = Codegen {
            target,
            emitter: match target {
                Target::ARM => Box::new(ARMEmitter::default()),
                Target::X86 => Box::new(X86Emitter::default()),
            },
            asm: AsmEmitter::default(),
        };

        codegen.asm.output
    }
}

impl Codegen {
    pub fn add_line(&mut self, line: &str) {
        self.asm.output.push_str(&format!("{line}\n"));
    }
}

#[derive(Debug, Clone, Default)]
pub struct AsmEmitter {
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct CodegenCtx<'a> {
    // codegen ctx is ephemeral
    pub func: &'a IRFunction,
    pub frame: &'a FrameLayout,
    pub block_labels: HashMap<BlockId, IRBlock>,
    pub current_block: BlockId,
    pub next_tmp_label: usize,
}

#[derive(Clone, Debug)]
pub struct FrameLayout {
    pub local_off: HashMap<usize, i32>,
    pub vreg_off: HashMap<VReg, i32>,
    pub frame_size: i32,
    pub align: i32,
}
