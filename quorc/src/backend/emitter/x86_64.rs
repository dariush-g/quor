use crate::{
    backend::{
        CodegenCtx, FrameLayout,
        lir::{
            regalloc::*,
            x86_64::{X86RegFpr, X86RegGpr, X86Regs},
        },
        target::TargetEmitter,
    },
    frontend::ast::Type,
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
        prologue.push_str(&format!("__q_f_{}:\n", func.name));
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
        _ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        match inst {
            LInst::Add { dst, a, b } => {
                format!(
                    "mov {}, {}\nadd {}, {}",
                    self.t_loc(dst.clone()),
                    self.t_operand(a),
                    self.t_loc(dst.clone()),
                    self.t_operand(b)
                )
            }
            LInst::Sub { dst, a, b } => {
                format!(
                    "mov {}, {}\nsub {}, {}",
                    self.t_loc(dst.clone()),
                    self.t_operand(a),
                    self.t_loc(dst.clone()),
                    self.t_operand(b)
                )
            }
            LInst::Mul { dst, a, b } => {
                format!(
                    "mov {}, {}\nimul {}, {}",
                    self.t_loc(dst.clone()),
                    self.t_operand(a),
                    self.t_loc(dst.clone()),
                    self.t_operand(b)
                )
            }
            LInst::Div { dst, a, b } => {
                format!(
                    "mov rax, {}\ncqo\nidiv {}\nmov {}, rax",
                    self.t_operand(a),
                    self.t_operand(b),
                    self.t_loc(dst.clone())
                )
            }
            LInst::Mod { dst, a, b } => {
                format!(
                    "mov rax, {}\ncqo\nidiv {}\nmov {}, rdx",
                    self.t_operand(a),
                    self.t_operand(b),
                    self.t_loc(dst.clone())
                )
            }
            LInst::CmpSet { dst, op, a, b } => {
                let setcc = match op {
                    CmpOp::Eq => "sete",
                    CmpOp::Ne => "setne",
                    CmpOp::Lt => "setl",
                    CmpOp::Le => "setle",
                    CmpOp::Gt => "setg",
                    CmpOp::Ge => "setge",
                };
                let dst_str = self.t_loc(dst.clone());
                format!(
                    "cmp {}, {}\n{} al\nmovzx eax, al\nmov {}, rax",
                    self.t_operand(a),
                    self.t_operand(b),
                    setcc,
                    dst_str
                )
            }
            LInst::Cast { dst, src, ty } => self.emit_cast(dst, src, ty),
            LInst::Load { dst, addr, ty } => self.emit_load(dst, addr, ty),
            LInst::Store { src, addr, ty } => self.emit_store(src, addr, ty),
            LInst::Call { dst, func, args } => self.emit_call(dst, func, args),
            LInst::Mov { dst, src } => {
                format!("mov {}, {}", self.t_loc(dst.clone()), self.t_operand(src))
            }
            LInst::Lea { dst, addr } => {
                format!("lea {}, {}", self.t_loc(dst.clone()), self.t_addr(addr.clone()))
            }
        }
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
        self.mem_ref_sized(&addr, "qword")
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

impl X86Emitter {
    fn type_size_suffix(ty: &Type) -> &'static str {
        match ty {
            Type::Char | Type::Bool => "byte",
            Type::int | Type::float => "dword",
            Type::Long | Type::Pointer(_) => "qword",
            _ => "qword",
        }
    }

    fn mem_ref_sized(&self, addr: &Addr<X86RegGpr>, size: &str) -> String {
        match addr {
            Addr::BaseOff { base, off } => {
                format!("{} [{} - {}]", size, self.target_args.reg64(*base), off)
            }
            Addr::BaseIndex {
                base,
                index,
                scale,
                off,
            } => format!(
                "{} [{} + {}*{} + {}]",
                size,
                self.target_args.reg64(*base),
                self.target_args.reg64(*index),
                scale,
                off
            ),
            Addr::Global { sym, off } => format!("{} [__q_g_{}+{}]", size, sym, off),
        }
    }

    fn emit_cast(
        &self,
        dst: &Loc<X86RegGpr, X86RegFpr>,
        src: &Operand<X86RegGpr, X86RegFpr>,
        ty: &Type,
    ) -> String {
        match ty {
            Type::Long => {
                format!(
                    "movsxd {}, {}",
                    self.t_loc(dst.clone()),
                    self.t_operand(src)
                )
            }
            Type::int => {
                format!(
                    "mov {}, {}",
                    self.t_loc(dst.clone()),
                    self.t_operand(src)
                )
            }
            _ => format!(
                "mov {}, {}",
                self.t_loc(dst.clone()),
                self.t_operand(src)
            ),
        }
    }

    fn emit_load(
        &self,
        dst: &Loc<X86RegGpr, X86RegFpr>,
        addr: &Addr<X86RegGpr>,
        ty: &Type,
    ) -> String {
        let size = Self::type_size_suffix(ty);
        let mem = self.mem_ref_sized(addr, size);
        match size {
            "byte" => format!("movzx rax, {}\nmov {}, rax", mem, self.t_loc(dst.clone())),
            "dword" => format!("mov eax, {}\nmov {}, rax", mem, self.t_loc(dst.clone())),
            _ => format!("mov rax, {}\nmov {}, rax", mem, self.t_loc(dst.clone())),
        }
    }

    fn emit_store(
        &self,
        src: &Operand<X86RegGpr, X86RegFpr>,
        addr: &Addr<X86RegGpr>,
        ty: &Type,
    ) -> String {
        let size = Self::type_size_suffix(ty);
        let mem = self.mem_ref_sized(addr, size);
        format!("mov {}, {}", mem, self.t_operand(src))
    }

    fn emit_call(
        &self,
        dst: &Option<Loc<X86RegGpr, X86RegFpr>>,
        func: &CallTarget<X86RegGpr>,
        args: &[Operand<X86RegGpr, X86RegFpr>],
    ) -> String {
        let arg_regs = self.target_args.arg_regs();
        let mut out = String::new();
        for (i, arg) in args.iter().take(6).enumerate() {
            out.push_str(&format!("mov {}, {}\n", arg_regs[i], self.t_operand(arg)));
        }
        let target = match func {
            CallTarget::Direct(sym) => format!("__q_f_{}", sym.0),
            CallTarget::Indirect(reg) => self.target_args.reg64(*reg).to_string(),
        };
        out.push_str(&format!("call {}", target));
        if let Some(d) = dst {
            out.push_str(&format!("\nmov {}, rax", self.t_loc(d.clone())));
        }
        out
    }
}
