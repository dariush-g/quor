use crate::{
    backend::{
        CodegenCtx, FrameLayout,
        lir::{
            regalloc::*,
            x86_64::{X86RegFpr, X86RegGpr, X86Regs},
        },
        target::TargetEmitter,
    },
    mir::block::*,
};

#[derive(Debug, Default)]
pub struct X86Emitter {
    target_args: X86Regs,
}

impl TargetEmitter for X86Emitter {
    type Reg = X86RegGpr;
    type FpReg = X86RegFpr;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        todo!()
    }

    fn t_prologue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        let mut prologue = String::new();
        prologue.push_str(&format!("{}:\n", func.name));
        prologue.push_str(&format!(
            "push rbp\nmov rbp, rsp\nsub rsp, {}",
            ctx.frame.frame_size
        ));
        prologue
    }

    fn t_epilogue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        let mut prologue = String::new();
        prologue.push_str(&format!(".Lret_{}\n", func.name));
        prologue.push_str(&format!("mov rsp, rbp\npop rbp",));
        prologue
    }
    
    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        String::new()
    }

    fn t_loc(&self, loc: crate::backend::lir::regalloc::Loc<Self::Reg, Self::FpReg>) -> String {
        match loc {
            Loc::PhysReg(reg_ref) => match reg_ref {
                RegRef::GprReg(r) => self.target_args.reg64(r).to_owned(),
                RegRef::FprReg(r) => self.target_args.float128(r).to_owned(),
            },
            Loc::Stack(offset) => format!("qword [rbx - {offset}]"),
        }
    }

    fn t_operand(&self, operand: &Operand<Self::Reg, Self::FpReg>) -> String {
        match operand {
            Operand::Loc(loc) => self.t_loc(loc.clone()),
            Operand::ImmI64(i) => i.to_string(),
            Operand::ImmF64(i) => i.to_string(),
            Operand::Indirect(addr) => self.t_addr(addr.clone()),
        }
    }

    fn t_addr(&self, addr: crate::backend::lir::regalloc::Addr<Self::Reg>) -> String {
        match addr {
            Addr::BaseOff { base, off } => {
                format!("qword [{} - {}]", self.target_args.reg64(base), off)
            }
            Addr::BaseIndex {
                base,
                index,
                scale,
                off,
            } => todo!(),
            Addr::Global { sym, off } => todo!(),
        }
    }

    fn t_emit_term(
        &mut self,
        term: &LTerm<Self::Reg, Self::FpReg>,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        todo!()
    }

    fn generate_ctx(
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> CodegenCtx<'_, Self::Reg, Self::FpReg> {
        let frame = FrameLayout {
            frame_size: (func.size + 15) & !15, // align up to 16
            align: 16,
        };

        CodegenCtx {
            func,
            frame,
            current_block: func.entry,
        }
    }
}
