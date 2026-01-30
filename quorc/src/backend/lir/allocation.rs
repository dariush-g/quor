use std::collections::HashMap;

use crate::{
    frontend::ast::Type,
    mir::block::{BlockId, IRFunction, VReg},
};

#[derive(Debug)]
pub enum Loc<R: Eq + std::fmt::Debug + std::hash::Hash> {
    PhysReg(R),
    Stack(i32),
    ImmI64(i64), // integer constant
    ImmF64(f64), // float constant
}

#[derive(Debug)]
pub enum Addr<R: Eq + std::hash::Hash + std::fmt::Debug> {
    BaseOff {
        base: Loc<R>,
        off: i32,
    }, // [base + off]
    BaseIndex {
        base: Loc<R>,
        index: Loc<R>,
        scale: u8,
        off: i32,
    }, // [base + index*scale + off]
    Global {
        sym: usize,
        off: i32,
    }, // materialize &global + off
}

#[derive(Debug)]
pub enum LInst<R: Eq + std::hash::Hash + std::fmt::Debug> {
    Add {
        dst: Loc<R>,
        a: Loc<R>,
        b: Loc<R>,
    },
    Sub {
        dst: Loc<R>,
        a: Loc<R>,
        b: Loc<R>,
    },
    Mul {
        dst: Loc<R>,
        a: Loc<R>,
        b: Loc<R>,
    },
    Div {
        dst: Loc<R>,
        a: Loc<R>,
        b: Loc<R>,
    },
    Mod {
        dst: Loc<R>,
        a: Loc<R>,
        b: Loc<R>,
    },

    CmpSet {
        dst: Loc<R>,
        op: CmpOp,
        a: Loc<R>,
        b: Loc<R>,
    },

    Cast {
        dst: Loc<R>,
        src: Loc<R>,
        ty: Type,
    },

    Load {
        dst: Loc<R>,
        addr: Addr<R>,
        ty: Type,
    },
    Store {
        src: Loc<R>,
        addr: Addr<R>,
        ty: Type,
    },

    Call {
        dst: Option<Loc<R>>,
        func: String,
        args: Vec<Loc<R>>,
    },

    Mov {
        dst: Loc<R>,
        src: Loc<R>,
    },
    Lea {
        dst: Loc<R>,
        addr: Addr<R>,
    }, // address-of in one op
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub trait TargetRegs {
    type Reg: Copy + Eq + std::fmt::Debug + std::hash::Hash;
    type FpReg: Copy + Eq + std::fmt::Debug + std::hash::Hash;

    fn all_regs() -> &'static [Self::Reg];
    fn allocatable_regs() -> &'static [Self::Reg];

    fn sp() -> Self::Reg;
    fn fp() -> Option<Self::Reg>;
    fn lr() -> Option<Self::Reg>; // aarch64 Some() | x86 None

    fn caller_saved_regs() -> &'static [Self::Reg];
    fn callee_saved_regs() -> &'static [Self::Reg];

    fn arg_regs() -> &'static [Self::Reg];
    fn ret_reg() -> Self::Reg;

    fn scratch_regs() -> &'static [Self::Reg];

    fn float_regs() -> &'static [Self::FpReg];

    fn is_caller_saved(r: Self::Reg) -> bool;
    fn is_callee_saved(r: Self::Reg) -> bool;

    fn fp_is_caller_saved(r: Self::FpReg) -> bool;
    fn fp_is_callee_saved(r: Self::FpReg) -> bool;

    fn reg32(reg: Self::Reg) -> &'static str;
    fn reg64(reg: Self::Reg) -> &'static str;

    fn float128(reg: Self::FpReg) -> &'static str;

    fn fp_caller_saved() -> &'static [Self::FpReg];
    fn fp_callee_saved() -> &'static [Self::FpReg];
}

pub struct Allocation<R: Copy + Eq + std::fmt::Debug + std::hash::Hash> {
    pub vreg_loc: HashMap<VReg, Loc<R>>,
    pub used_callee_saved: Vec<R>,
}

pub struct LFunction<R: Copy + Eq + std::fmt::Debug + std::hash::Hash> {
    pub name: String,
    pub blocks: Vec<LBlock<R>>,
    pub entry: BlockId,
}

pub struct LBlock<R: Copy + Eq + std::fmt::Debug + std::hash::Hash> {
    pub id: BlockId,
    pub insts: Vec<LInst<R>>,
    pub term: LTerm<R>,
}

pub enum LTerm<R: Copy + Eq + std::fmt::Debug + std::hash::Hash> {
    Ret {
        value: Option<Loc<R>>,
    },
    Jump {
        target: BlockId,
    },
    Branch {
        condition: Loc<R>,
        if_true: BlockId,
        if_false: BlockId,
    },
}

pub trait RegAlloc<R: Copy + Eq + std::fmt::Debug + std::hash::Hash> {
    fn allocate(func: &IRFunction) -> LFunction<R> {
        unimplemented!()
    }
}
