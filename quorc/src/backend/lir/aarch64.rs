use crate::backend::lir::allocation::TargetRegs;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum A64RegGpr {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    FP, // X29
    LR, // X30
    SP,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum A64RegFpr {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
    V18,
    V19,
    V20,
    V21,
    V22,
    V23,
    V24,
    V25,
    V26,
    V27,
    V28,
    V29,
    V30,
    V31,
}

impl TargetRegs for A64RegGpr {
    type Reg = A64RegGpr;
    type FpReg = A64RegFpr;

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

    fn is_caller_saved(r: Self::Reg) -> bool {
        todo!()
    }

    fn is_callee_saved(r: Self::Reg) -> bool {
        todo!()
    }

    fn reg32(reg: Self::Reg) -> &'static str {
        todo!()
    }

    fn reg64(reg: Self::Reg) -> &'static str {
        todo!()
    }

    fn float_regs() -> &'static [Self::FpReg] {
        todo!()
    }
}
