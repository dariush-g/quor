use std::collections::HashMap;

use crate::{
    backend::{
        aarch64::ARMEmitter,
        target::{Target, TargetEmitter},
        x86_64::X86Emitter,
    },
    mir::block::*,
};

pub mod aarch64;
pub mod target;
pub mod x86_64;
pub mod x86_64_;
pub mod lir;


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

        let mut codegen = Codegen {
            target,
            emitter: match target {
                Target::ARM => Box::new(ARMEmitter::default()),
                Target::X86 => Box::new(X86Emitter::default()),
            },
            asm: AsmEmitter::default(),
        };

        for constant in ir_program.global_consts {
            let constant_ = codegen.emitter.t_add_global_const(constant.clone());
            if let GlobalValue::String(_) = constant.value {
                codegen.add_line(AsmSection::CSTRING, &constant_);
            } else {
                codegen.add_line(AsmSection::RODATA, &constant_);
            }
        }

        for (_, function) in ir_program.functions {
            codegen.emitter.generate_function(&function);
        }

        codegen.emit()
    }

    pub fn add_line(&mut self, section: AsmSection, line: &str) {
        match section {
            AsmSection::BSS => self.asm.bss.push_str(&format!("{line}\n")),
            AsmSection::RODATA => self.asm.rodata.push_str(&format!("{line}\n")),
            AsmSection::DATA => self.asm.data.push_str(&format!("{line}\n")),
            AsmSection::TEXT => self.asm.text.push_str(&format!("{line}\n")),
            AsmSection::CSTRING => self.asm.cstrings.push_str(&format!("{line}\n")),
        }
    }

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    fn emit(&self) -> String {
        let rodata = format!(".section __TEXT,__const\n{}", self.asm.rodata);
        let cstrings = format!(".section __TEXT,__cstring\n{}", self.asm.cstrings);
        let data = format!(".section __DATA,__data\n{}", self.asm.data);
        let bss = format!(".section __DATA,__bss\n{}", self.asm.bss);
        let text = format!(".section __TEXT,__text\n{}", self.asm.text);

        format!("{cstrings}{rodata}{data}{bss}{text}")
    }

    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn emit(&self) -> String {
        let rodata = format!(".section .rodata\n{}", self.asm.rodata);
        let data = format!(".section .data\n{}", self.asm.data);
        let bss = format!(".section .bss\n{}", self.asm.bss);
        let text = format!(".section .text\n{}", self.asm.text);

        format!("{rodata}{data}{bss}{text}")
    }

    #[cfg(target_arch = "x86_64")]
    fn emit(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AsmEmitter {
    pub text: String,
    pub data: String,
    pub rodata: String,
    pub bss: String,
    #[cfg(target_os = "macos")]
    pub cstrings: String,
}

#[derive(Clone, Debug, Copy)]
pub enum AsmSection {
    BSS,
    RODATA,
    DATA,
    TEXT,
    #[cfg(target_os = "macos")]
    CSTRING,
}

#[derive(Clone, Debug)]
pub struct CodegenCtx<'a> {
    // codegen ctx is ephemeral
    pub func: &'a IRFunction,
    pub frame: &'a FrameLayout,
    pub block_labels: HashMap<BlockId, &'a IRBlock>,
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
