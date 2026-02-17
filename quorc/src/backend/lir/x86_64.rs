use crate::backend::lir::regalloc::TargetRegs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86RegGpr {
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86RegFpr {
    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM8,
    XMM9,
    XMM10,
    XMM11,
    XMM12,
    XMM13,
    XMM14,
    XMM15,
}

impl X86RegFpr {
    pub const ARG_REGS: &'static [X86RegFpr] = &[
        X86RegFpr::XMM0,
        X86RegFpr::XMM1,
        X86RegFpr::XMM2,
        X86RegFpr::XMM3,
        X86RegFpr::XMM4,
        X86RegFpr::XMM5,
        X86RegFpr::XMM6,
        X86RegFpr::XMM7,
    ];

    pub const ALL: &'static [X86RegFpr] = &[
        X86RegFpr::XMM0,
        X86RegFpr::XMM1,
        X86RegFpr::XMM2,
        X86RegFpr::XMM3,
        X86RegFpr::XMM4,
        X86RegFpr::XMM5,
        X86RegFpr::XMM6,
        X86RegFpr::XMM7,
        X86RegFpr::XMM8,
        X86RegFpr::XMM9,
        X86RegFpr::XMM10,
        X86RegFpr::XMM11,
        X86RegFpr::XMM12,
        X86RegFpr::XMM13,
        X86RegFpr::XMM14,
        X86RegFpr::XMM15,
    ];

    pub const FP_CALLER_SAVED: &'static [X86RegFpr] = &[
        X86RegFpr::XMM0,
        X86RegFpr::XMM1,
        X86RegFpr::XMM2,
        X86RegFpr::XMM3,
        X86RegFpr::XMM4,
        X86RegFpr::XMM5,
        X86RegFpr::XMM6,
        X86RegFpr::XMM7,
    ];

    pub const FP_CALLEE_SAVED: &'static [X86RegFpr] = &[
        X86RegFpr::XMM8,
        X86RegFpr::XMM9,
        X86RegFpr::XMM10,
        X86RegFpr::XMM11,
        X86RegFpr::XMM12,
        X86RegFpr::XMM13,
        X86RegFpr::XMM14,
        X86RegFpr::XMM15,
    ];
}

impl X86RegGpr {
    pub const ARG_REGS: &'static [X86RegGpr] = &[
        X86RegGpr::RDI,
        X86RegGpr::RSI,
        X86RegGpr::RDX,
        X86RegGpr::RCX,
        X86RegGpr::R8,
        X86RegGpr::R9,
    ];

    pub const ALL: &'static [X86RegGpr] = &[
        X86RegGpr::RAX,
        X86RegGpr::RBX,
        X86RegGpr::RCX,
        X86RegGpr::RDX,
        X86RegGpr::RSI,
        X86RegGpr::RDI,
        X86RegGpr::RBP,
        X86RegGpr::RSP,
        X86RegGpr::R8,
        X86RegGpr::R9,
        X86RegGpr::R10,
        X86RegGpr::R11,
        X86RegGpr::R12,
        X86RegGpr::R13,
        X86RegGpr::R14,
        X86RegGpr::R15,
    ];

    pub const GPR_CALLER_SAVED: &'static [X86RegGpr] = &[
        X86RegGpr::RAX,
        X86RegGpr::RCX,
        X86RegGpr::RDX,
        X86RegGpr::RSI,
        X86RegGpr::RDI,
        X86RegGpr::R8,
        X86RegGpr::R9,
        X86RegGpr::R10,
        X86RegGpr::R11,
    ];

    pub const GPR_CALLEE_SAVED: &'static [X86RegGpr] = &[
        X86RegGpr::RBX,
        X86RegGpr::RBP,
        X86RegGpr::R12,
        X86RegGpr::R13,
        X86RegGpr::R14,
        X86RegGpr::R15,
    ];

    pub const ALLOCATABLE: &'static [X86RegGpr] = &[
        X86RegGpr::RAX,
        X86RegGpr::RCX,
        X86RegGpr::RDX,
        X86RegGpr::RSI,
        X86RegGpr::RDI,
        X86RegGpr::R8,
        X86RegGpr::R9,
        X86RegGpr::R10,
        X86RegGpr::R11,
        X86RegGpr::RBX,
        X86RegGpr::R12,
        X86RegGpr::R13,
        X86RegGpr::R14,
        X86RegGpr::R15,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct X86Regs;

impl TargetRegs for X86Regs {
    const NUM_ALLOCATABLE: usize = 14;
    const FPR_ALLOCATABLE: usize = 15;

    type Reg = X86RegGpr;
    type FpReg = X86RegFpr;

    fn all_regs(&self) -> &'static [Self::Reg] {
        X86RegGpr::ALL
    }

    fn allocatable_regs(&self) -> &'static [Self::Reg] {
        X86RegGpr::ALLOCATABLE
    }

    fn sp(&self) -> Self::Reg {
        X86RegGpr::RSP
    }

    fn fp(&self) -> Option<Self::Reg> {
        None
    }

    fn lr(&self) -> Option<Self::Reg> {
        None
    }

    fn caller_saved_regs(&self) -> &'static [Self::Reg] {
        X86RegGpr::GPR_CALLER_SAVED
    }

    fn callee_saved_regs(&self) -> &'static [Self::Reg] {
        X86RegGpr::GPR_CALLEE_SAVED
    }

    fn arg_regs(&self) -> &'static [Self::Reg] {
        X86RegGpr::ARG_REGS
    }

    fn ret_reg(&self) -> Self::Reg {
        X86RegGpr::RAX
    }

    fn scratch_regs(&self) -> &'static [Self::Reg] {
        todo!()
    }

    fn float_regs(&self) -> &'static [Self::FpReg] {
        X86RegFpr::ALL
    }

    fn is_caller_saved(&self, r: &Self::Reg) -> bool {
        matches!(
            r,
            X86RegGpr::RAX
                | X86RegGpr::RCX
                | X86RegGpr::RDX
                | X86RegGpr::RSI
                | X86RegGpr::RDI
                | X86RegGpr::R8
                | X86RegGpr::R9
                | X86RegGpr::R10
                | X86RegGpr::R11
        )
    }

    fn is_callee_saved(&self, r: &Self::Reg) -> bool {
        matches!(
            r,
            X86RegGpr::RBX
                | X86RegGpr::RBP
                | X86RegGpr::R12
                | X86RegGpr::R13
                | X86RegGpr::R14
                | X86RegGpr::R15
        )
    }

    fn fp_is_caller_saved(&self, r: Self::FpReg) -> bool {
        matches!(
            r,
            X86RegFpr::XMM0
                | X86RegFpr::XMM1
                | X86RegFpr::XMM2
                | X86RegFpr::XMM3
                | X86RegFpr::XMM4
                | X86RegFpr::XMM5
                | X86RegFpr::XMM6
                | X86RegFpr::XMM7
        )
    }

    fn fp_is_callee_saved(&self, r: Self::FpReg) -> bool {
        matches!(
            r,
            X86RegFpr::XMM8
                | X86RegFpr::XMM9
                | X86RegFpr::XMM10
                | X86RegFpr::XMM11
                | X86RegFpr::XMM12
                | X86RegFpr::XMM13
                | X86RegFpr::XMM14
                | X86RegFpr::XMM15
        )
    }

    fn reg32(&self, reg: Self::Reg) -> &'static str {
        match reg {
            X86RegGpr::RAX => "eax",
            X86RegGpr::RBX => "ebx",
            X86RegGpr::RCX => "ecx",
            X86RegGpr::RDX => "edx",
            X86RegGpr::RSI => "esi",
            X86RegGpr::RDI => "edi",
            X86RegGpr::RBP => "ebp",
            X86RegGpr::RSP => "esp",
            X86RegGpr::R8 => "r8d",
            X86RegGpr::R9 => "r9d",
            X86RegGpr::R10 => "r10d",
            X86RegGpr::R11 => "r11d",
            X86RegGpr::R12 => "r12d",
            X86RegGpr::R13 => "r13d",
            X86RegGpr::R14 => "r14d",
            X86RegGpr::R15 => "r15d",
        }
    }

    fn reg64(&self, reg: Self::Reg) -> &'static str {
        match reg {
            X86RegGpr::RAX => "rax",
            X86RegGpr::RBX => "rbx",
            X86RegGpr::RCX => "rcx",
            X86RegGpr::RDX => "rdx",
            X86RegGpr::RSI => "rsi",
            X86RegGpr::RDI => "rdi",
            X86RegGpr::RBP => "rbp",
            X86RegGpr::RSP => "rsp",
            X86RegGpr::R8 => "r8",
            X86RegGpr::R9 => "r9",
            X86RegGpr::R10 => "r10",
            X86RegGpr::R11 => "r11",
            X86RegGpr::R12 => "r12",
            X86RegGpr::R13 => "r13",
            X86RegGpr::R14 => "r14",
            X86RegGpr::R15 => "r15",
        }
    }

    fn float128(&self, reg: Self::FpReg) -> &'static str {
        match reg {
            X86RegFpr::XMM0 => "xmm0",
            X86RegFpr::XMM1 => "xmm1",
            X86RegFpr::XMM2 => "xmm2",
            X86RegFpr::XMM3 => "xmm3",
            X86RegFpr::XMM4 => "xmm4",
            X86RegFpr::XMM5 => "xmm5",
            X86RegFpr::XMM6 => "xmm6",
            X86RegFpr::XMM7 => "xmm7",
            X86RegFpr::XMM8 => "xmm8",
            X86RegFpr::XMM9 => "xmm9",
            X86RegFpr::XMM10 => "xmm10",
            X86RegFpr::XMM11 => "xmm11",
            X86RegFpr::XMM12 => "xmm12",
            X86RegFpr::XMM13 => "xmm13",
            X86RegFpr::XMM14 => "xmm14",
            X86RegFpr::XMM15 => "xmm15",
        }
    }

    fn fp_caller_saved(&self) -> &'static [Self::FpReg] {
        X86RegFpr::FP_CALLER_SAVED
    }

    fn fp_callee_saved(&self) -> &'static [Self::FpReg] {
        X86RegFpr::FP_CALLEE_SAVED
    }

    fn fp_arg_regs(&self) -> &'static [Self::FpReg] {
        X86RegFpr::ARG_REGS
    }
}
