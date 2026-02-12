use std::{collections::HashMap, hash::Hash};

use crate::{
    backend::lir::{SymId, aarch64::A64RegGpr},
    frontend::ast::Type,
    mir::block::{BlockId, GlobalDef, GlobalValue, IRFunction, IRProgram, VReg},
};

pub fn mir_to_lir(mir: IRProgram) {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RegRef<
    R: Eq + Copy + std::fmt::Debug + Copy + Hash,
    F: Eq + Copy + std::fmt::Debug + Copy + Hash,
> {
    GprReg(R),
    FprReg(F),
}

#[derive(Debug, Clone)]
pub enum Loc<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    PhysReg(RegRef<R, F>),
    Stack(i32),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

pub trait TargetRegs
where
    <Self as TargetRegs>::Reg: 'static,
    <Self as TargetRegs>::FpReg: 'static,
{
    type Reg: Copy + Eq + std::fmt::Debug + std::hash::Hash;
    type FpReg: Copy + Eq + std::fmt::Debug + std::hash::Hash;

    fn all_regs(&self) -> &'static [Self::Reg];
    fn allocatable_regs(&self) -> &'static [Self::Reg];

    fn sp(&self) -> Self::Reg;
    fn fp(&self) -> Option<Self::Reg>;
    fn lr(&self) -> Option<Self::Reg>; // aarch64 Some() | x86 None

    fn caller_saved_regs(&self) -> &'static [Self::Reg];
    fn callee_saved_regs(&self) -> &'static [Self::Reg];

    fn arg_regs(&self) -> &'static [Self::Reg];
    fn fp_arg_regs(&self) -> &'static [Self::FpReg];
    fn ret_reg(&self) -> Self::Reg;

    fn scratch_regs(&self) -> &'static [Self::Reg];

    fn float_regs(&self) -> &'static [Self::FpReg];

    fn is_caller_saved(&self, r: Self::Reg) -> bool;
    fn is_callee_saved(&self, r: Self::Reg) -> bool;

    fn fp_is_caller_saved(&self, r: Self::FpReg) -> bool;
    fn fp_is_callee_saved(&self, r: Self::FpReg) -> bool;

    fn reg32(&self, reg: Self::Reg) -> &'static str;
    fn reg64(&self, reg: Self::Reg) -> &'static str;

    fn float128(&self, reg: Self::FpReg) -> &'static str;

    fn fp_caller_saved(&self) -> &'static [Self::FpReg];
    fn fp_callee_saved(&self) -> &'static [Self::FpReg];

    fn regalloc(&self, func: &IRFunction) -> Allocation<Self::Reg, Self::FpReg> {
        let mut vreg_loc: HashMap<
            VReg,
            Loc<<Self as TargetRegs>::Reg, <Self as TargetRegs>::FpReg>,
        > = HashMap::new();
        let used_callee_saved = Vec::new();
        let used_callee_saved_fp = Vec::new();

        // TODO: Fp reg params

        for (param, reg) in func.params.clone().iter().zip(self.arg_regs()) {
            vreg_loc.insert(*param, Loc::PhysReg(RegRef::GprReg(*reg)));
        }

        println!("{:?}", vreg_loc);

        Allocation {
            vreg_loc,
            used_callee_saved,
            used_callee_saved_fp,
        }
    }

    fn to_lir<
        R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
        F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    >(
        &self,
        func: &IRFunction,
    ) -> LFunction<R, F> {
        let name = func.name.clone();
        let allocation = self.regalloc(func);

        LFunction {
            name,
            blocks: todo!(),
            entry: todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Allocation<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub vreg_loc: HashMap<VReg, Loc<R, F>>,
    pub used_callee_saved: Vec<R>,
    pub used_callee_saved_fp: Vec<R>,
}

#[derive(Debug, Clone)]
pub struct LFunction<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub name: String,
    pub blocks: Vec<LBlock<R, F>>,
    pub entry: BlockId,
}

#[derive(Debug, Clone)]
pub struct LBlock<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub id: BlockId,
    pub insts: Vec<LInst<R, F>>,
    pub term: LTerm<R, F>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct LProgram<
    R: Copy + Eq + std::fmt::Debug + std::hash::Hash,
    F: Copy + Eq + std::fmt::Debug + std::hash::Hash,
> {
    pub functions: Vec<LFunction<R, F>>,
    // pub structs: Vec<LStructDef>,
    pub globals: Vec<LGlobalDef>,
}

#[derive(Debug, Clone)]
pub struct LStructDef {
    pub name: String,
    pub fields: HashMap<String, (i32, Type)>,
    pub is_union: bool,
}

#[derive(Debug, Clone)]
pub struct LGlobalDef {
    pub id: usize,
    pub value: GlobalValue,
}
