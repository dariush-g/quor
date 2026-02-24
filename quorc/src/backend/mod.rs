#[cfg(target_arch = "x86_64")]
use crate::{backend::lir::x86_64::X86Regs, emitter::x86_64::X86Emitter};
use crate::{
    backend::{
        emitter::x86_64::X86Emitter,
        lir::{
            regalloc::{LFunction, TargetRegs},
            x86_64::X86Regs,
        },
    },
    target::{target_arch, target_os},
};

use crate::{
    backend::{emitter::aarch64::ARMEmitter, lir::aarch64::A64Regs, target::TargetEmitter},
    midend::mir::block::*,
};

pub mod emitter;
pub mod lir;
pub mod target;

#[derive(Debug)]
pub struct Codegen {
    pub asm: AsmEmitter,
    pub target_codegen: TargetCodegen,
}

#[derive(Debug)]
pub enum TargetCodegen {
    X86(X86Codegen),
    Arm(Arm64Codegen),
}

impl TargetCodegen {
    fn get_emitter_x86(&self) -> &X86Emitter {
        match self {
            TargetCodegen::X86(x86_codegen) => &x86_codegen.emitter,
            _ => panic!(),
        }
    }

    fn get_emitter_arm(&mut self) -> &ARMEmitter {
        match self {
            TargetCodegen::Arm(arm_codegen) => &arm_codegen.emitter,
            _ => panic!(),
        }
    }

    fn get_target_regs_x86(&self) -> &X86Regs {
        match self {
            TargetCodegen::X86(x86_codegen) => &x86_codegen.target_regs,
            _ => panic!(),
        }
    }

    fn get_target_regs_arm(&self) -> &A64Regs {
        match self {
            TargetCodegen::Arm(arm_codegen) => &arm_codegen.target_regs,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Default)]
pub struct X86Codegen {
    pub emitter: X86Emitter,
    pub target_regs: X86Regs,
}

#[derive(Debug, Default)]
pub struct Arm64Codegen {
    pub emitter: ARMEmitter,
    pub target_regs: A64Regs,
}

impl Codegen {
    pub fn generate(ir_program: IRProgram) -> String {
        let mut codegen = Codegen {
            asm: AsmEmitter::default(),
            target_codegen: match target_arch() {
                "x86_64" => TargetCodegen::X86(X86Codegen::default()),
                "aarch64" => TargetCodegen::Arm(Arm64Codegen::default()),
                _ => panic!(),
            },
        };

        for externed in ir_program.externs {
            match target_arch() {
                "x86_64" => {
                    let emitted = codegen.target_codegen.get_emitter_x86().t_extern(externed);
                    codegen.add_line(AsmSection::EXTERN, &emitted);
                }
                "aarch64" => {
                    let emitted = codegen.target_codegen.get_emitter_arm().t_extern(externed);
                    codegen.add_line(AsmSection::EXTERN, &emitted);
                }
                _ => panic!(),
            }
        }

        for constant in ir_program.global_consts {
            let constant_ = match target_arch() {
                "x86_64" => {
                    let emitter = codegen.target_codegen.get_emitter_x86();
                    emitter.t_add_global_const(constant.clone())
                }
                "aarch64" => codegen
                    .target_codegen
                    .get_emitter_arm()
                    .t_add_global_const(constant.clone()),
                _ => panic!(),
            };
            if let GlobalValue::String(_) = constant.value
                && target_os() == "macos"
            {
                codegen.add_line(AsmSection::CSTRING, &constant_);
            } else {
                codegen.add_line(AsmSection::RODATA, &constant_);
            }
        }

        for (_, func) in ir_program.functions {
            match target_arch() {
                "x86_64" => {
                    let lir = &codegen.target_codegen.get_target_regs_x86().to_lir(&func);
                    let func = &codegen
                        .target_codegen
                        .get_emitter_x86()
                        .generate_function(lir);
                    codegen.asm.text.push_str(func);
                }
                "aarch64" => {
                    let lir = &codegen.target_codegen.get_target_regs_arm().to_lir(&func);
                    let func = codegen
                        .target_codegen
                        .get_emitter_arm()
                        .generate_function(lir);
                    codegen.asm.text.push_str(&func);
                }
                _ => panic!(),
            };
        }

        match (target_arch(), target_os()) {
            ("x86_64", _) => codegen.emit_x86_64(),
            (_, "macos") => codegen.emit_macos(),
            _ => codegen.emit_aarch64_linux(),
        }
    }

    pub fn add_line(&mut self, section: AsmSection, line: &str) {
        match section {
            AsmSection::BSS => self.asm.bss.push_str(&format!("{line}\n")),
            AsmSection::RODATA => self.asm.rodata.push_str(&format!("{line}\n")),
            AsmSection::DATA => self.asm.data.push_str(&format!("{line}\n")),
            AsmSection::TEXT => self.asm.text.push_str(&format!("{line}\n")),
            AsmSection::CSTRING => self.asm.cstrings.push_str(&format!("{line}\n")),
            AsmSection::EXTERN => self.asm.externs.push_str(&format!("{line}\n")),
        }
    }

    fn emit_macos(&self) -> String {
        let externs = format!("{}\n", self.asm.externs);
        let rodata = format!(".section __TEXT,__const\n{}", self.asm.rodata);
        let cstrings = format!(".section __TEXT,__cstring\n{}", self.asm.cstrings);
        let data = format!(".section __DATA,__data\n{}", self.asm.data);
        let bss = format!(".section __DATA,__bss\n{}", self.asm.bss);
        let text = format!(".section __TEXT,__text\n{}", self.asm.text);

        format!("{externs}{cstrings}{rodata}{data}{bss}{text}")
    }

    fn emit_aarch64_linux(&self) -> String {
        let externs = format!("{}\n", self.asm.externs);
        let rodata = format!(".section .rodata\n{}", self.asm.rodata);
        let data = format!(".section .data\n{}", self.asm.data);
        let bss = format!(".section .bss\n{}", self.asm.bss);
        let text = format!(".section .text\n{}", self.asm.text);

        format!("{externs}{rodata}{data}{bss}{text}")
    }

    fn emit_x86_64(&self) -> String {
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
    pub cstrings: String,
}

#[derive(Clone, Debug, Copy)]
pub enum AsmSection {
    EXTERN,
    BSS,
    RODATA,
    DATA,
    TEXT,
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
