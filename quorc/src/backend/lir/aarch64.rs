use crate::backend::lir::regalloc::TargetRegs;

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

impl A64RegFpr {
    pub const ARG_REGS: &'static [A64RegFpr] = &[
        A64RegFpr::V0,
        A64RegFpr::V1,
        A64RegFpr::V2,
        A64RegFpr::V3,
        A64RegFpr::V4,
        A64RegFpr::V5,
        A64RegFpr::V6,
        A64RegFpr::V7,
    ];

    pub const FP_CALLER_SAVED: &'static [A64RegFpr] = &[
        A64RegFpr::V0,
        A64RegFpr::V1,
        A64RegFpr::V2,
        A64RegFpr::V3,
        A64RegFpr::V4,
        A64RegFpr::V5,
        A64RegFpr::V6,
        A64RegFpr::V7,
        A64RegFpr::V16,
        A64RegFpr::V17,
        A64RegFpr::V18,
        A64RegFpr::V19,
        A64RegFpr::V20,
        A64RegFpr::V21,
        A64RegFpr::V22,
        A64RegFpr::V23,
        A64RegFpr::V24,
        A64RegFpr::V25,
        A64RegFpr::V26,
        A64RegFpr::V27,
        A64RegFpr::V28,
        A64RegFpr::V29,
        A64RegFpr::V30,
        A64RegFpr::V31,
    ];

    pub const ALL_FP: &'static [A64RegFpr] = &[
        A64RegFpr::V0,
        A64RegFpr::V1,
        A64RegFpr::V2,
        A64RegFpr::V3,
        A64RegFpr::V4,
        A64RegFpr::V5,
        A64RegFpr::V6,
        A64RegFpr::V7,
        A64RegFpr::V8,
        A64RegFpr::V9,
        A64RegFpr::V10,
        A64RegFpr::V11,
        A64RegFpr::V12,
        A64RegFpr::V13,
        A64RegFpr::V14,
        A64RegFpr::V15,
        A64RegFpr::V16,
        A64RegFpr::V17,
        A64RegFpr::V18,
        A64RegFpr::V19,
        A64RegFpr::V20,
        A64RegFpr::V21,
        A64RegFpr::V22,
        A64RegFpr::V23,
        A64RegFpr::V24,
        A64RegFpr::V25,
        A64RegFpr::V26,
        A64RegFpr::V27,
        A64RegFpr::V28,
        A64RegFpr::V29,
        A64RegFpr::V30,
        A64RegFpr::V31,
    ];

    const FP_CALLEE_SAVED: &'static [A64RegFpr] = &[
        A64RegFpr::V8,
        A64RegFpr::V9,
        A64RegFpr::V10,
        A64RegFpr::V11,
        A64RegFpr::V12,
        A64RegFpr::V13,
        A64RegFpr::V14,
        A64RegFpr::V15,
    ];
}

impl A64RegGpr {
    pub const ARG_REGS: &'static [A64RegGpr] = &[
        A64RegGpr::X0,
        A64RegGpr::X1,
        A64RegGpr::X2,
        A64RegGpr::X3,
        A64RegGpr::X4,
        A64RegGpr::X5,
        A64RegGpr::X6,
        A64RegGpr::X7,
    ];

    pub const ALL: &'static [A64RegGpr] = &[
        A64RegGpr::X0,
        A64RegGpr::X1,
        A64RegGpr::X2,
        A64RegGpr::X3,
        A64RegGpr::X4,
        A64RegGpr::X5,
        A64RegGpr::X6,
        A64RegGpr::X7,
        A64RegGpr::X8,
        A64RegGpr::X9,
        A64RegGpr::X10,
        A64RegGpr::X11,
        A64RegGpr::X12,
        A64RegGpr::X13,
        A64RegGpr::X14,
        A64RegGpr::X15,
        A64RegGpr::X16,
        A64RegGpr::X17,
        A64RegGpr::X18,
        A64RegGpr::X19,
        A64RegGpr::X20,
        A64RegGpr::X21,
        A64RegGpr::X22,
        A64RegGpr::X23,
        A64RegGpr::X24,
        A64RegGpr::X25,
        A64RegGpr::X26,
        A64RegGpr::X27,
        A64RegGpr::X28,
        A64RegGpr::FP,
        A64RegGpr::LR,
        A64RegGpr::SP,
    ];

    pub const ALLOCATABLE: &'static [A64RegGpr] = &[
        A64RegGpr::X19,
        A64RegGpr::X20,
        A64RegGpr::X21,
        A64RegGpr::X22,
        A64RegGpr::X23,
        A64RegGpr::X24,
        A64RegGpr::X25,
        A64RegGpr::X26,
        A64RegGpr::X27,
        A64RegGpr::X28,
    ];

    pub const CALLER_SAVED: &'static [A64RegGpr] = &[
        A64RegGpr::X0,
        A64RegGpr::X1,
        A64RegGpr::X2,
        A64RegGpr::X3,
        A64RegGpr::X4,
        A64RegGpr::X5,
        A64RegGpr::X6,
        A64RegGpr::X7,
        A64RegGpr::X8,
        A64RegGpr::X9,
        A64RegGpr::X10,
        A64RegGpr::X11,
        A64RegGpr::X12,
        A64RegGpr::X13,
        A64RegGpr::X14,
        A64RegGpr::X15,
        A64RegGpr::X16,
        A64RegGpr::X17,
    ];
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
        A64RegGpr::ALL
    }

    fn allocatable_regs() -> &'static [Self::Reg] {
        A64RegGpr::ALLOCATABLE
    }

    fn sp() -> Self::Reg {
        A64RegGpr::SP
    }

    fn fp() -> Option<Self::Reg> {
        Some(A64RegGpr::FP)
    }

    fn lr() -> Option<Self::Reg> {
        Some(A64RegGpr::LR)
    }

    fn caller_saved_regs() -> &'static [Self::Reg] {
        A64RegGpr::CALLER_SAVED
    }

    fn callee_saved_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn arg_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn ret_reg() -> Self::Reg {
        A64RegGpr::X0
    }

    fn scratch_regs() -> &'static [Self::Reg] {
        todo!()
    }

    fn is_caller_saved(r: Self::Reg) -> bool {
        matches!(
            r,
            A64RegGpr::X0
                | A64RegGpr::X1
                | A64RegGpr::X2
                | A64RegGpr::X3
                | A64RegGpr::X4
                | A64RegGpr::X5
                | A64RegGpr::X6
                | A64RegGpr::X7
                | A64RegGpr::X8
                | A64RegGpr::X9
                | A64RegGpr::X10
                | A64RegGpr::X11
                | A64RegGpr::X12
                | A64RegGpr::X13
                | A64RegGpr::X14
                | A64RegGpr::X15
                | A64RegGpr::X16
                | A64RegGpr::X17
        )
    }

    fn is_callee_saved(r: Self::Reg) -> bool {
        matches!(
            r,
            A64RegGpr::X19
                | A64RegGpr::X20
                | A64RegGpr::X21
                | A64RegGpr::X22
                | A64RegGpr::X23
                | A64RegGpr::X24
                | A64RegGpr::X25
                | A64RegGpr::X26
                | A64RegGpr::X27
                | A64RegGpr::X28
        )
    }

    fn reg32(reg: Self::Reg) -> &'static str {
        match reg {
            A64RegGpr::X0 => "w0",
            A64RegGpr::X1 => "w1",
            A64RegGpr::X2 => "w2",
            A64RegGpr::X3 => "w3",
            A64RegGpr::X4 => "w4",
            A64RegGpr::X5 => "w5",
            A64RegGpr::X6 => "w6",
            A64RegGpr::X7 => "w7",
            A64RegGpr::X8 => "w8",
            A64RegGpr::X9 => "w9",
            A64RegGpr::X10 => "w10",
            A64RegGpr::X11 => "w11",
            A64RegGpr::X12 => "w12",
            A64RegGpr::X13 => "w13",
            A64RegGpr::X14 => "w14",
            A64RegGpr::X15 => "w15",
            A64RegGpr::X16 => "w16",
            A64RegGpr::X17 => "w17",
            A64RegGpr::X18 => "w18",
            A64RegGpr::X19 => "w19",
            A64RegGpr::X20 => "w20",
            A64RegGpr::X21 => "w21",
            A64RegGpr::X22 => "w22",
            A64RegGpr::X23 => "w23",
            A64RegGpr::X24 => "w24",
            A64RegGpr::X25 => "w25",
            A64RegGpr::X26 => "w26",
            A64RegGpr::X27 => "w27",
            A64RegGpr::X28 => "w28",
            A64RegGpr::FP => "w29",
            A64RegGpr::LR => "w30",
            A64RegGpr::SP => "sp",
        }
    }

    fn reg64(reg: Self::Reg) -> &'static str {
        match reg {
            A64RegGpr::X0 => "x0",
            A64RegGpr::X1 => "x1",
            A64RegGpr::X2 => "x2",
            A64RegGpr::X3 => "x3",
            A64RegGpr::X4 => "x4",
            A64RegGpr::X5 => "x5",
            A64RegGpr::X6 => "x6",
            A64RegGpr::X7 => "x7",
            A64RegGpr::X8 => "x8",
            A64RegGpr::X9 => "x9",
            A64RegGpr::X10 => "x10",
            A64RegGpr::X11 => "x11",
            A64RegGpr::X12 => "x12",
            A64RegGpr::X13 => "x13",
            A64RegGpr::X14 => "x14",
            A64RegGpr::X15 => "x15",
            A64RegGpr::X16 => "x16",
            A64RegGpr::X17 => "x17",
            A64RegGpr::X18 => "x18",
            A64RegGpr::X19 => "x19",
            A64RegGpr::X20 => "x20",
            A64RegGpr::X21 => "x21",
            A64RegGpr::X22 => "x22",
            A64RegGpr::X23 => "x23",
            A64RegGpr::X24 => "x24",
            A64RegGpr::X25 => "x25",
            A64RegGpr::X26 => "x26",
            A64RegGpr::X27 => "x27",
            A64RegGpr::X28 => "x28",
            A64RegGpr::FP => "x29",
            A64RegGpr::LR => "x30",
            A64RegGpr::SP => "sp",
        }
    }

    fn float_regs() -> &'static [Self::FpReg] {
        A64RegFpr::ALL_FP
    }

    fn float128(reg: Self::FpReg) -> &'static str {
        match reg {
            A64RegFpr::V0 => "v0",
            A64RegFpr::V1 => "v1",
            A64RegFpr::V2 => "v2",
            A64RegFpr::V3 => "v3",
            A64RegFpr::V4 => "v4",
            A64RegFpr::V5 => "v5",
            A64RegFpr::V6 => "v6",
            A64RegFpr::V7 => "v7",
            A64RegFpr::V8 => "v8",
            A64RegFpr::V9 => "v9",
            A64RegFpr::V10 => "v10",
            A64RegFpr::V11 => "v11",
            A64RegFpr::V12 => "v12",
            A64RegFpr::V13 => "v13",
            A64RegFpr::V14 => "v14",
            A64RegFpr::V15 => "v15",
            A64RegFpr::V16 => "v16",
            A64RegFpr::V17 => "v17",
            A64RegFpr::V18 => "v18",
            A64RegFpr::V19 => "v19",
            A64RegFpr::V20 => "v20",
            A64RegFpr::V21 => "v21",
            A64RegFpr::V22 => "v22",
            A64RegFpr::V23 => "v23",
            A64RegFpr::V24 => "v24",
            A64RegFpr::V25 => "v25",
            A64RegFpr::V26 => "v26",
            A64RegFpr::V27 => "v27",
            A64RegFpr::V28 => "v28",
            A64RegFpr::V29 => "v29",
            A64RegFpr::V30 => "v30",
            A64RegFpr::V31 => "v31",
        }
    }

    fn fp_is_caller_saved(r: Self::FpReg) -> bool {
        matches!(
            r,
            A64RegFpr::V0
                | A64RegFpr::V1
                | A64RegFpr::V2
                | A64RegFpr::V3
                | A64RegFpr::V4
                | A64RegFpr::V5
                | A64RegFpr::V6
                | A64RegFpr::V7
                | A64RegFpr::V16
                | A64RegFpr::V17
                | A64RegFpr::V18
                | A64RegFpr::V19
                | A64RegFpr::V20
                | A64RegFpr::V21
                | A64RegFpr::V22
                | A64RegFpr::V23
                | A64RegFpr::V24
                | A64RegFpr::V25
                | A64RegFpr::V26
                | A64RegFpr::V27
                | A64RegFpr::V28
                | A64RegFpr::V29
                | A64RegFpr::V30
                | A64RegFpr::V31
        )
    }

    fn fp_is_callee_saved(r: Self::FpReg) -> bool {
        matches!(
            r,
            A64RegFpr::V8
                | A64RegFpr::V9
                | A64RegFpr::V10
                | A64RegFpr::V11
                | A64RegFpr::V12
                | A64RegFpr::V13
                | A64RegFpr::V14
                | A64RegFpr::V15
        )
    }

    fn fp_caller_saved() -> &'static [Self::FpReg] {
        A64RegFpr::FP_CALLER_SAVED
    }

    fn fp_callee_saved() -> &'static [Self::FpReg] {
        A64RegFpr::FP_CALLEE_SAVED
    }

    fn fp_arg_regs() -> &'static [Self::FpReg] {
        A64RegFpr::ARG_REGS
    }
}
