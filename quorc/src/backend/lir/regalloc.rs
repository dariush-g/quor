use std::{collections::HashMap, hash::Hash};

use crate::{
    backend::lir::{SymId, aarch64::A64RegGpr},
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
}

#[derive(Debug)]
pub enum Operand<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    Loc(Loc<R, F>),
    ImmI64(i64), // integer constant
    ImmF64(f64), // float constant}
}

impl<R: Copy + Eq + Hash + std::fmt::Debug, F: Copy + Eq + Hash + std::fmt::Debug> From<Loc<R, F>>
    for Operand<R, F>
{
    fn from(p: Loc<R, F>) -> Self {
        Operand::Loc(p)
    }
}

#[derive(Debug)]
pub enum Addr<R: Copy + Eq + std::hash::Hash + std::fmt::Debug> {
    BaseOff {
        base: R,
        off: i32,
    }, // [base + off]
    BaseIndex {
        base: R,
        index: R,
        scale: u8,
        off: i32,
    }, // [base + index*scale + off]
    Global {
        sym: usize,
        off: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallTarget<R> {
    Direct(SymId),
    Indirect(R),
}

#[derive(Debug)]
pub enum LInst<R: Copy + Eq + Hash + std::fmt::Debug, F: Copy + Eq + Hash + std::fmt::Debug> {
    Add {
        dst: Loc<R, F>,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },
    Sub {
        dst: Loc<R, F>,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },
    Mul {
        dst: Loc<R, F>,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },
    Div {
        dst: Loc<R, F>,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },
    Mod {
        dst: Loc<R, F>,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },
    CmpSet {
        dst: Loc<R, F>,
        op: CmpOp,
        a: Operand<R, F>,
        b: Operand<R, F>,
    },

    Cast {
        dst: Loc<R, F>,
        src: Operand<R, F>,
        ty: Type,
    },

    // Memory
    Load {
        dst: Loc<R, F>,
        addr: Addr<R>,
        ty: Type,
    },
    Store {
        src: Operand<R, F>,
        addr: Addr<R>,
        ty: Type,
    },

    // Calls
    Call {
        dst: Option<Loc<R, F>>,
        func: CallTarget<R>,
        args: Vec<Operand<R, F>>,
    },

    // Move / lea
    Mov {
        dst: Loc<R, F>,
        src: Operand<R, F>,
    },
    Lea {
        dst: Loc<R, F>,
        addr: Addr<R>,
    },
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
    fn fp_arg_regs() -> &'static [Self::FpReg];
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

    fn regalloc() -> Allocation<Self::Reg, Self::FpReg> {
        let vreg_loc = HashMap::new();
        let used_callee_saved = Vec::new();
        let used_callee_saved_fp = Vec::new();

        

        Allocation {
            vreg_loc,
            used_callee_saved,
            used_callee_saved_fp,
        }
    }
}

pub struct Allocation<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub vreg_loc: HashMap<VReg, Loc<R, F>>,
    pub used_callee_saved: Vec<R>,
    pub used_callee_saved_fp: Vec<R>,
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
        value: Option<Operand<R, F>>,
    },
    Jump {
        target: BlockId,
    },
    Branch {
        condition: Operand<R, F>,
        if_true: BlockId,
        if_false: BlockId,
    },
}
