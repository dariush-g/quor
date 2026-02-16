use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    backend::lir::{
        SymId,
        aarch64::A64RegGpr,
        interference::{InterferenceGraph, build_interference_graph},
    },
    frontend::ast::Type,
    mir::block::{
        BlockId, GlobalDef, GlobalValue, IRBlock, IRFunction, IRInstruction, IRProgram, Terminator,
        VReg, VRegType, Value,
    },
};

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
    const NUM_ALLOCATABLE: usize;
    const FPR_ALLOCATABLE: usize;

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

        let gp_args = self.arg_regs();
        let fp_args = self.fp_arg_regs();

        let (int_params, float_params): (Vec<_>, Vec<_>) = func
            .params
            .iter()
            .partition(|p| matches!(p.ty, VRegType::Int));

        // Assign integer params
        vreg_loc.extend(
            int_params
                .iter()
                .zip(gp_args.iter())
                .map(|(param, &reg)| (*param, Loc::PhysReg(RegRef::GprReg(reg)))),
        );

        // Assign float params
        vreg_loc.extend(
            float_params
                .iter()
                .zip(fp_args.iter())
                .map(|(param, &reg)| (*param, Loc::PhysReg(RegRef::FprReg(reg)))),
        );

        let flattened_blocks = flatten_blocks(func.blocks.clone());
        let live_ranges = assign_live_ranges(flattened_blocks);

        let (gpr_ranges, fpr_ranges): (HashMap<_, _>, HashMap<_, _>) = live_ranges
            .into_iter()
            .partition(|(vreg, _)| matches!(vreg.ty, VRegType::Int));

        let gpr_graph = build_interference_graph(&gpr_ranges);
        let fpr_graph = build_interference_graph(&fpr_ranges);

        let mut gpr_stack = Vec::new();
        let mut fpr_stack = Vec::new();

        for (node, connections) in gpr_graph.edges.clone() {
            if connections.len() < Self::NUM_ALLOCATABLE {
                gpr_stack.push(node);
            }
        }

        for (node, connections) in gpr_graph.edges.clone() {
            if connections.len() > Self::NUM_ALLOCATABLE {
                gpr_stack.push(node);
            }
        }

        for (node, connections) in fpr_graph.edges.clone() {
            if connections.len() < Self::FPR_ALLOCATABLE {
                fpr_stack.push(node);
            }
        }

        for (node, connections) in fpr_graph.edges.clone() {
            if connections.len() > Self::FPR_ALLOCATABLE {
                fpr_stack.push(node);
            }
        }

        let gpr_alloc = self.color_graph_gpr(gpr_stack, &gpr_graph);
        let fpr_alloc = self.color_graph_fpr(fpr_stack, &fpr_graph);

        println!("{gpr_alloc:?}");
        println!("{fpr_alloc:?}");

        Allocation {
            vreg_loc,
            used_callee_saved,
            used_callee_saved_fp,
        }
    }

    fn color_graph_gpr(
        &self,
        stack: Vec<VReg>,
        graph: &InterferenceGraph,
    ) -> HashMap<VReg, Loc<Self::Reg, Self::FpReg>> {
        let mut post_prologue_stack_offset = 0;
        let mut allocation: HashMap<VReg, Loc<Self::Reg, Self::FpReg>> = HashMap::new();

        for node in stack.into_iter().rev() {
            let neighbors = graph.neighbors(&node);

            let neighbor_colors: HashSet<usize> = neighbors
                .iter()
                .filter_map(|n| {
                    if let Some(Loc::PhysReg(RegRef::GprReg(reg))) = allocation.get(n) {
                        self.allocatable_regs().iter().position(|r| r == reg)
                    } else {
                        None
                    }
                })
                .collect();

            if let Some(color) = (0..Self::NUM_ALLOCATABLE).find(|c| !neighbor_colors.contains(c)) {
                let phys_reg = self.allocatable_regs()[color];
                allocation.insert(node, Loc::PhysReg(RegRef::GprReg(phys_reg)));
            } else {
                allocation.insert(node, Loc::Stack(post_prologue_stack_offset));
                post_prologue_stack_offset += 8;
            }
        }

        allocation
    }

    fn color_graph_fpr(
        &self,
        stack: Vec<VReg>,
        graph: &InterferenceGraph,
    ) -> HashMap<VReg, Loc<Self::Reg, Self::FpReg>> {
        let mut post_prologue_stack_offset = 0;
        let mut allocation: HashMap<VReg, Loc<Self::Reg, Self::FpReg>> = HashMap::new();

        for node in stack.into_iter().rev() {
            let neighbors = graph.neighbors(&node);

            let neighbor_colors: HashSet<usize> = neighbors
                .iter()
                .filter_map(|n| {
                    if let Some(Loc::PhysReg(RegRef::FprReg(reg))) = allocation.get(n) {
                        self.float_regs().iter().position(|r| r == reg)
                    } else {
                        None
                    }
                })
                .collect();

            if let Some(color) = (0..Self::NUM_ALLOCATABLE).find(|c| !neighbor_colors.contains(c)) {
                let phys_reg = self.float_regs()[color];
                allocation.insert(node, Loc::PhysReg(RegRef::FprReg(phys_reg)));
            } else {
                allocation.insert(node, Loc::Stack(post_prologue_stack_offset));
                post_prologue_stack_offset += 8;
            }
        }

        allocation
    }

    fn find_node_with_degree_less_than_k(&self, graph: &InterferenceGraph) -> Vec<VReg> {
        let mut ret = Vec::new();
        for (node, connections) in graph.edges.clone() {
            match node.ty {
                VRegType::Int => {
                    if connections.len() < Self::NUM_ALLOCATABLE {
                        ret.push(node);
                    }
                }
                VRegType::Float => {
                    if connections.len() < Self::FPR_ALLOCATABLE {
                        ret.push(node);
                    }
                }
            }
        }
        ret
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

fn flatten_blocks(blocks: Vec<IRBlock>) -> Vec<LifetimeInstr> {
    let mut insts = Vec::new();

    for block in blocks {
        insts.extend(
            block
                .instructions
                .into_iter()
                .map(LifetimeInstr::IRInstruction),
        );
        insts.push(LifetimeInstr::Terminator(block.terminator));
    }

    insts
}

#[derive(Debug, Clone)]
pub enum LifetimeInstr {
    IRInstruction(IRInstruction),
    Terminator(Terminator),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct LiveRange {
    pub vreg: VReg,
    pub start: usize, // index of start and end within the Vec<LifetimeInstr>
    pub end: usize,
}

fn vreg_of_value(value: &Value) -> Option<&VReg> {
    if let Value::Reg(vreg) = value {
        return Some(vreg);
    }
    None
}
fn assign_live_ranges(insts: Vec<LifetimeInstr>) -> HashMap<VReg, LiveRange> {
    let mut map: HashMap<VReg, LiveRange> = HashMap::new();

    for (idx, inst) in insts.iter().enumerate() {
        match inst {
            LifetimeInstr::IRInstruction(irinstruction) => match irinstruction {
                IRInstruction::Add { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Sub { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Mul { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Div { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Mod { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Eq { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Ne { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Lt { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Le { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Ge { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Gt { reg, left, right } => {
                    update_live_range(vreg_of_value(right), &mut map, idx);
                    update_live_range(vreg_of_value(left), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Cast { reg, src, .. } => {
                    update_live_range(vreg_of_value(src), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Load { reg, addr, .. } => {
                    update_live_range(vreg_of_value(addr), &mut map, idx);
                    update_live_range(Some(reg), &mut map, idx);
                }
                IRInstruction::Store { value, addr, .. } => {
                    update_live_range(vreg_of_value(addr), &mut map, idx);
                    update_live_range(vreg_of_value(value), &mut map, idx);
                }
                IRInstruction::Gep { dest, base, .. } => {
                    update_live_range(Some(dest), &mut map, idx);
                    update_live_range(vreg_of_value(base), &mut map, idx);
                }
                IRInstruction::Call { reg, args, .. } => {
                    update_live_range(reg.as_ref(), &mut map, idx);
                    args.iter()
                        .for_each(|arg| update_live_range(vreg_of_value(arg), &mut map, idx));
                }
                IRInstruction::Move { dest, from } => {
                    update_live_range(Some(dest), &mut map, idx);
                    update_live_range(vreg_of_value(from), &mut map, idx);
                }
                IRInstruction::AddressOf { dest, src } => {
                    update_live_range(Some(dest), &mut map, idx);
                    update_live_range(vreg_of_value(src), &mut map, idx);
                }
                IRInstruction::Memcpy { dst, src, .. } => {
                    update_live_range(vreg_of_value(src), &mut map, idx);
                    update_live_range(vreg_of_value(dst), &mut map, idx);
                }
                _ => {}
            },
            LifetimeInstr::Terminator(terminator) => match terminator {
                Terminator::Return { value: Some(value) } => {
                    update_live_range(vreg_of_value(value), &mut map, idx)
                }
                Terminator::Branch { condition, .. } => {
                    update_live_range(vreg_of_value(condition), &mut map, idx)
                }
                _ => {}
            },
        }
    }

    map
}

fn update_live_range(vreg: Option<&VReg>, map: &mut HashMap<VReg, LiveRange>, idx: usize) {
    if let Some(vreg) = vreg {
        map.entry(*vreg)
            .and_modify(|range| range.end = idx)
            .or_insert(LiveRange {
                vreg: *vreg,
                start: idx,
                end: idx,
            });
    }
}
