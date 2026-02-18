#[cfg(target_arch = "x86_64")]
use crate::backend::lir::x86_64::X86Regs;
use crate::backend::lir::{
    aarch64::A64RegFpr,
    regalloc::{LFunction, TargetRegs},
};
use std::collections::HashMap;

use crate::{
    backend::{
        emitter::{aarch64::ARMEmitter, x86_64::X86Emitter},
        lir::aarch64::A64Regs,
        target::{Target, TargetEmitter},
    },
    mir::block::*,
};

pub mod emitter;
pub mod lir;
pub mod target;
pub mod x86_64_;

#[derive(Debug)]
pub struct Codegen {
    pub target: Target,
    #[cfg(target_arch = "aarch64")]
    pub emitter: ARMEmitter,
    #[cfg(target_arch = "x86_64")]
    pub emitter: X86Emitter,
    #[cfg(target_arch = "aarch64")]
    pub target_regs: A64Regs,
    #[cfg(target_arch = "x86_64")]
    pub target_regs: X86Regs,
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
            #[cfg(target_arch = "aarch64")]
            emitter: ARMEmitter::default(),
            #[cfg(target_arch = "x86_64")]
            emitter: X86Emitter::default(),
            #[cfg(target_arch = "aarch64")]
            target_regs: A64Regs,
            #[cfg(target_arch = "x86_64")]
            target_regs: X86Regs,
            asm: AsmEmitter::default(),
        };

        for externed in ir_program.externs {
            codegen.add_line(AsmSection::EXTERN, &codegen.emitter.t_extern(externed));
        }

        for constant in ir_program.global_consts {
            let constant_ = codegen.emitter.t_add_global_const(constant.clone());
            if let GlobalValue::String(_) = constant.value
                && cfg!(target_os = "macos")
            {
                #[cfg(target_os = "macos")]
                codegen.add_line(AsmSection::CSTRING, &constant_);
            } else {
                codegen.add_line(AsmSection::RODATA, &constant_);
            }
        }

        for (_, func) in ir_program.functions {
            codegen.asm.text.push_str(
                &codegen
                    .emitter
                    .generate_function(&codegen.target_regs.to_lir(&func)),
            );
        }

        codegen.emit()
    }

    pub fn add_line(&mut self, section: AsmSection, line: &str) {
        match section {
            AsmSection::BSS => self.asm.bss.push_str(&format!("{line}\n")),
            AsmSection::RODATA => self.asm.rodata.push_str(&format!("{line}\n")),
            AsmSection::DATA => self.asm.data.push_str(&format!("{line}\n")),
            AsmSection::TEXT => self.asm.text.push_str(&format!("{line}\n")),
            #[cfg(target_os = "macos")]
            AsmSection::CSTRING => self.asm.cstrings.push_str(&format!("{line}\n")),
            AsmSection::EXTERN => self.asm.externs.push_str(&format!("{line}\n")),
        }
    }

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    fn emit(&self) -> String {
        let externs = format!("{}\n", self.asm.externs);
        let rodata = format!(".section __TEXT,__const\n{}", self.asm.rodata);
        let cstrings = format!(".section __TEXT,__cstring\n{}", self.asm.cstrings);
        let data = format!(".section __DATA,__data\n{}", self.asm.data);
        let bss = format!(".section __DATA,__bss\n{}", self.asm.bss);
        let text = format!(".section __TEXT,__text\n{}", self.asm.text);

        format!("{externs}{cstrings}{rodata}{data}{bss}{text}")
    }

    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn emit(&self) -> String {
        let externs = format!("{}\n", self.asm.externs);
        let rodata = format!(".section .rodata\n{}", self.asm.rodata);
        let data = format!(".section .data\n{}", self.asm.data);
        let bss = format!(".section .bss\n{}", self.asm.bss);
        let text = format!(".section .text\n{}", self.asm.text);

        format!("{externs}{rodata}{data}{bss}{text}")
    }

    #[cfg(target_arch = "x86_64")]
    fn emit(&self) -> String {
        let rodata = format!("section .rodata\n{}", self.asm.rodata);
        let data = format!("section .data\n{}", self.asm.data);
        let bss = format!("section .bss\n{}", self.asm.bss);
        let text = format!("section .text\n{}", self.asm.text);
        let externs = format!("{}\n", self.asm.externs);

        format!("{externs}{rodata}{data}{bss}{text}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct AsmEmitter {
    pub externs: String,
    pub text: String,
    pub data: String,
    pub rodata: String,
    pub bss: String,
    #[cfg(target_os = "macos")]
    pub cstrings: String,
}

#[derive(Clone, Debug, Copy)]
pub enum AsmSection {
    EXTERN,
    BSS,
    RODATA,
    DATA,
    TEXT,
    #[cfg(target_os = "macos")]
    CSTRING,
}

#[derive(Clone, Debug)]
pub struct CodegenCtx<'a, R, F>
where
    R: Copy + Clone + std::fmt::Debug + std::hash::Hash + Eq,
    F: Copy + Clone + std::fmt::Debug + std::hash::Hash + Eq,
{
    // codegen ctx is ephemeral
    pub func: &'a LFunction<R, F>,
    pub frame: FrameLayout,
    pub current_block: BlockId,
}

#[derive(Clone, Debug)]
pub struct FrameLayout {
    pub frame_size: i32,
    pub align: i32,
}
