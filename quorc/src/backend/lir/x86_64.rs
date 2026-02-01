use crate::backend::lir::allocation::TargetRegs;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct X86Regs;

impl TargetRegs for X86Regs {
    type Reg = X86RegGpr;
    type FpReg = X86RegFpr;

    fn all_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn allocatable_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn sp() -> Self::Reg {
        todo!()
    }

    fn fp() -> Option<Self::Reg> {
        todo!()
    }

    fn lr() -> Option<Self::Reg> {
        todo!()
    }

    fn caller_saved_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn callee_saved_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn arg_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn ret_reg() -> Self::Reg {
        todo!()
    }

    fn scratch_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn float_regs() -> &'static [Self::FpReg] {
        todo!()
    }

    fn is_caller_saved(r: Self::Reg) -> bool {
        todo!()
    }

    fn is_callee_saved(r: Self::Reg) -> bool {
        todo!()
    }

    fn fp_is_caller_saved(r: Self::FpReg) -> bool {
        todo!()
    }

    fn fp_is_callee_saved(r: Self::FpReg) -> bool {
        todo!()
    }

    fn reg32(reg: Self::Reg) -> &'static str {
        todo!()
    }

    fn reg64(reg: Self::Reg) -> &'static str {
        todo!()
    }

    fn float128(reg: Self::FpReg) -> &'static str {
        todo!()
    }

    fn fp_caller_saved() -> &'static [Self::FpReg] {
        todo!()
    }

    fn fp_callee_saved() -> &'static [Self::FpReg] {
        todo!()
    }
}
