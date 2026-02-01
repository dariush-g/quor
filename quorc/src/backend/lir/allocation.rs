use std::{collections::HashMap, hash::Hash};

use crate::{
    frontend::ast::Type,
    mir::block::{BlockId, IRFunction, VReg},
};

impl IRFunction {
    pub fn to_lir<
        R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
        F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    >(
        &self,
    ) -> LFunction<R, F> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RegRef<
    R: Eq + Copy + std::fmt::Debug + Copy + Hash,
    F: Eq + Copy + std::fmt::Debug + Copy + Hash,
> {
    GprReg(R),
    FprReg(F),
}

#[derive(Debug)]
pub enum Loc<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    PhysReg(RegRef<R, F>),
    Stack(i32),
    ImmI64(i64), // integer constant
    ImmF64(f64), // float constant
}

#[derive(Debug)]
pub enum Addr<
    R: Copy + Eq + std::hash::Hash + std::fmt::Debug,
    F: Copy + Eq + std::hash::Hash + std::fmt::Debug,
> {
    BaseOff {
        base: Loc<R, F>,
        off: i32,
    }, // [base + off]
    BaseIndex {
        base: Loc<R, F>,
        index: Loc<R, F>,
        scale: u8,
        off: i32,
    }, // [base + index*scale + off]
    Global {
        sym: usize,
        off: i32,
    }, // materialize &global + off
}

#[derive(Debug)]
pub enum LInst<
    R: Eq + std::hash::Hash + std::fmt::Debug + Copy,
    F: Eq + std::hash::Hash + std::fmt::Debug + Copy,
> {
    Add {
        dst: Loc<R, F>,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },
    Sub {
        dst: Loc<R, F>,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },
    Mul {
        dst: Loc<R, F>,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },
    Div {
        dst: Loc<R, F>,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },
    Mod {
        dst: Loc<R, F>,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },

    CmpSet {
        dst: Loc<R, F>,
        op: CmpOp,
        a: Loc<R, F>,
        b: Loc<R, F>,
    },

    Cast {
        dst: Loc<R, F>,
        src: Loc<R, F>,
        ty: Type,
    },

    Load {
        dst: Loc<R, F>,
        addr: Addr<R, F>,
        ty: Type,
    },
    Store {
        src: Loc<R, F>,
        addr: Addr<R, F>,
        ty: Type,
    },

    Call {
        dst: Option<Loc<R, F>>,
        func: String,
        args: Vec<Loc<R, F>>,
    },

    Mov {
        dst: Loc<R, F>,
        src: Loc<R, F>,
    },
    Lea {
        dst: Loc<R, F>,
        addr: Addr<R, F>,
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

pub struct Allocation<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub vreg_loc: HashMap<VReg, Loc<R, F>>,
    pub used_callee_saved: Vec<R>,
}

pub struct LFunction<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub name: String,
    pub blocks: Vec<LBlock<R, F>>,
    pub entry: BlockId,
}

pub struct LBlock<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub id: BlockId,
    pub insts: Vec<LInst<R, F>>,
    pub term: LTerm<R, F>,
}

pub enum LTerm<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    Ret {
        value: Option<Loc<R, F>>,
    },
    Jump {
        target: BlockId,
    },
    Branch {
        condition: Loc<R, F>,
        if_true: BlockId,
        if_false: BlockId,
    },
}
