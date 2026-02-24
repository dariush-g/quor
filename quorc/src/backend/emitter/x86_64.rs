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
    midend::mir::block::*,
};

#[derive(Debug, Default)]
pub struct X86Emitter {
    target_args: X86Regs,
}

impl TargetEmitter for X86Emitter {
    type Reg = X86RegGpr;
    type FpReg = X86RegFpr;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        let symbol = format!("__q_g_{}", constant.id);

        let mut out = String::new();
        out.push_str(&format!("{}:\n", symbol));

        match &constant.value {
            GlobalValue::Int(v) => {
                out.push_str(&format!("    dq {}\n", v));
            }

            GlobalValue::Float(f) => {
                out.push_str(&format!("    dq 0x{:016x}\n", f.to_bits()));
            }

            GlobalValue::Bool(b) => {
                let val = if *b { 1 } else { 0 };
                out.push_str(&format!("    db {}\n", val));
            }

            GlobalValue::Char(c) => {
                out.push_str(&format!("    db {}\n", *c as u8));
            }

            GlobalValue::String(s) => {
                out.push_str(&format!("    db {:?}, 0\n", s));
            }

            GlobalValue::Bytes(bytes) => {
                let list = bytes
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("    db {}\n", list));
            }

            GlobalValue::Zeroed(size) => {
                out.push_str(&format!("    times {} db 0\n", size));
            }

            GlobalValue::Array(values) => {
                for val in values {
                    match val {
                        GlobalValue::Int(v) => {
                            out.push_str(&format!("    dq {}\n", v));
                        }
                        GlobalValue::Bool(b) => {
                            let val = if *b { 1 } else { 0 };
                            out.push_str(&format!("    db {}\n", val));
                        }
                        GlobalValue::Char(c) => {
                            out.push_str(&format!("    db {}\n", *c as u8));
                        }
                        _ => unimplemented!("Nested array type not implemented"),
                    }
                }
            }

            GlobalValue::Struct(_) => {
                unimplemented!("Struct global emission not implemented yet");
            }
        }

        out
    }

    fn t_prologue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &crate::backend::lir::regalloc::LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        let mut prologue = String::new();
        if func.name != "main" {
            prologue.push_str(&format!("__q_f_{}:\n", func.name));
        } else {
            prologue.push_str(&format!("global main\n{}:\n", func.name));
        }
        prologue.push_str(&format!(
            "push rbp\nmov rbp, rsp\nsub rsp, {}\n",
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
        prologue.push_str(&format!(".Lret_{}:\n", func.name));
        prologue.push_str("mov rsp, rbp\npop rbp\nret\n");
        prologue
    }

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        _ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        match inst {
            LInst::Add { dst, a, b } => {
                let w = Self::loc_width(dst);
                format!(
                    "mov {}, {}\nadd {}, {}\n",
                    self.t_loc_at(dst, w),
                    self.t_operand_at(a, w),
                    self.t_loc_at(dst, w),
                    self.t_operand_at(b, w)
                )
            }
            LInst::Sub { dst, a, b } => {
                let w = Self::loc_width(dst);
                format!(
                    "mov {}, {}\nsub {}, {}\n",
                    self.t_loc_at(dst, w),
                    self.t_operand_at(a, w),
                    self.t_loc_at(dst, w),
                    self.t_operand_at(b, w)
                )
            }
            LInst::Mul { dst, a, b } => {
                let w = Self::loc_width(dst);
                format!(
                    "mov {}, {}\nimul {}, {}\n",
                    self.t_loc_at(dst, w),
                    self.t_operand_at(a, w),
                    self.t_loc_at(dst, w),
                    self.t_operand_at(b, w)
                )
            }
            LInst::Div { dst, a, b } => {
                let w = Self::loc_width(dst);
                let rax = self.target_args.reg_by_width(X86RegGpr::RAX, w);
                let sign_ext = match w {
                    RegWidth::W8 => "cbw",
                    RegWidth::W16 => "cwd",
                    RegWidth::W32 => "cdq",
                    _ => "cqo",
                };
                format!(
                    "mov {}, {}\n{}\nidiv {}\nmov {}, {}\n",
                    rax,
                    self.t_operand_at(a, w),
                    sign_ext,
                    self.t_operand_at(b, w),
                    self.t_loc_at(dst, w),
                    rax
                )
            }
            LInst::Mod { dst, a, b } => {
                let w = Self::loc_width(dst);
                let rax = self.target_args.reg_by_width(X86RegGpr::RAX, w);
                let rdx = self.target_args.reg_by_width(X86RegGpr::RDX, w);
                let sign_ext = match w {
                    RegWidth::W8 => "cbw",
                    RegWidth::W16 => "cwd",
                    RegWidth::W32 => "cdq",
                    _ => "cqo",
                };
                format!(
                    "mov {}, {}\n{}\nidiv {}\nmov {}, {}\n",
                    rax,
                    self.t_operand_at(a, w),
                    sign_ext,
                    self.t_operand_at(b, w),
                    self.t_loc_at(dst, w),
                    rdx
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
                // cmp operands must match each other's width
                let cmp_w = Self::operand_width(a);
                let dst_w = Self::loc_width(dst);
                format!(
                    "cmp {}, {}\n{} al\nmovzx {}, al\n",
                    self.t_operand_at(a, cmp_w),
                    self.t_operand_at(b, cmp_w),
                    setcc,
                    self.t_loc_at(dst, dst_w),
                )
            }
            LInst::Cast { dst, src, ty } => self.emit_cast(dst, src, ty),
            LInst::Load { dst, addr, ty } => self.emit_load(dst, addr, ty),
            LInst::Store { src, addr, ty } => self.emit_store(src, addr, ty),
            LInst::Call { dst, func, args } => self.emit_call(dst, func, args),
            LInst::Mov { dst, src } => {
                let w = Self::loc_width(dst);
                format!("mov {}, {}\n", self.t_loc_at(dst, w), self.t_operand_at(src, w))
            }
            LInst::Lea { dst, addr } => {
                // LEA always operates on addresses (64-bit)
                format!(
                    "lea {}, {}\n",
                    self.t_loc_at(dst, RegWidth::W64),
                    self.t_addr(addr.clone())
                )
            }
            LInst::InlineAsm { asm } => format!("{}\n", asm),
        }
    }

    fn t_loc(&self, loc: crate::backend::lir::regalloc::Loc<Self::Reg, Self::FpReg>) -> String {
        match loc {
            Loc::PhysReg(rr) => match &rr.ty {
                RegType::GprReg(r) => self.target_args.reg_by_width(*r, rr.size).to_owned(),
                RegType::FprReg(r) => self.target_args.float128(*r).to_owned(),
            },
            Loc::Stack(offset) => format!("qword [rbp - {offset}]\n"),
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
        let mut asm = String::new();
        match term {
            LTerm::Ret { value } => {
                if let Some(operand) = value {
                    let w = Self::operand_width(operand);
                    let rax = self.target_args.reg_by_width(X86RegGpr::RAX, w);
                    asm.push_str(&format!("mov {}, {}\n", rax, self.t_operand_at(operand, w)));
                }
                asm.push_str(&format!("jmp .Lret_{}\n", ctx.func.name));
            }
            LTerm::Jump { target } => todo!(),
            LTerm::Branch {
                condition,
                if_true,
                if_false,
            } => todo!(),
        };
        asm
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

    fn t_extern(&self, ext: String) -> String {
        format!("extern {ext}\n")
    }
}

impl X86Emitter {
    fn width_to_size_prefix(w: RegWidth) -> &'static str {
        match w {
            RegWidth::W8 => "byte",
            RegWidth::W16 => "word",
            RegWidth::W32 => "dword",
            RegWidth::W64 | RegWidth::W128 => "qword",
        }
    }

    fn loc_width(loc: &Loc<X86RegGpr, X86RegFpr>) -> RegWidth {
        match loc {
            Loc::PhysReg(rr) => rr.size,
            Loc::Stack(_) => RegWidth::W64,
        }
    }

    fn operand_width(op: &Operand<X86RegGpr, X86RegFpr>) -> RegWidth {
        match op {
            Operand::Loc(loc) => Self::loc_width(loc),
            _ => RegWidth::W64,
        }
    }

    /// Render a Loc at a specific register width.
    fn t_loc_at(&self, loc: &Loc<X86RegGpr, X86RegFpr>, w: RegWidth) -> String {
        match loc {
            Loc::PhysReg(rr) => match &rr.ty {
                RegType::GprReg(r) => self.target_args.reg_by_width(*r, w).to_owned(),
                RegType::FprReg(r) => self.target_args.float128(*r).to_owned(),
            },
            Loc::Stack(offset) => {
                let size = Self::width_to_size_prefix(w);
                format!("{} [rbp - {}]", size, offset)
            }
        }
    }

    /// Render an Operand at a specific register width.
    fn t_operand_at(&self, operand: &Operand<X86RegGpr, X86RegFpr>, w: RegWidth) -> String {
        match operand {
            Operand::Loc(loc) => self.t_loc_at(loc, w),
            Operand::ImmI64(i) => i.to_string(),
            Operand::ImmF64(i) => i.to_string(),
            Operand::Indirect(addr) => self.t_addr(addr.clone()),
        }
    }

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
            Addr::Global { sym, off } => {
                if *off == 0 {
                    format!("{} [rel __q_g_{}]", size, sym)
                } else {
                    format!("{} [rel __q_g_{}+{}]", size, sym, off)
                }
            }
        }
    }

    fn emit_cast(
        &self,
        dst: &Loc<X86RegGpr, X86RegFpr>,
        src: &Operand<X86RegGpr, X86RegFpr>,
        ty: &Type,
    ) -> String {
        let dst_w = Self::loc_width(dst);
        match ty {
            Type::Long | Type::Pointer(_) => {
                // Sign-extend to 64-bit: movsxd uses 32-bit source, 64-bit dest
                format!(
                    "movsxd {}, {}\n",
                    self.t_loc_at(dst, RegWidth::W64),
                    self.t_operand_at(src, RegWidth::W32)
                )
            }
            Type::Bool | Type::Char => {
                // Zero-extend byte to dst width
                format!(
                    "movzx {}, {}\n",
                    self.t_loc_at(dst, dst_w),
                    self.t_operand_at(src, RegWidth::W8)
                )
            }
            Type::int => {
                // Truncate or same-size move at 32-bit
                format!("mov {}, {}\n", self.t_loc_at(dst, RegWidth::W32), self.t_operand_at(src, RegWidth::W32))
            }
            _ => {
                // Default: move at dst width
                format!("mov {}, {}\n", self.t_loc_at(dst, dst_w), self.t_operand_at(src, dst_w))
            }
        }
    }

    fn emit_load(
        &self,
        dst: &Loc<X86RegGpr, X86RegFpr>,
        addr: &Addr<X86RegGpr>,
        ty: &Type,
    ) -> String {
        let size = Self::type_size_suffix(ty);
        let w = Self::loc_width(dst);
        let rax = self.target_args.reg_by_width(X86RegGpr::RAX, w);
        let mem = self.mem_ref_sized(addr, size);
        let mut ret = match size {
            "byte" => format!("movzx {}, {}\n", rax, mem),
            "dword" => format!("mov {}, {}\n", self.target_args.reg_by_width(X86RegGpr::RAX, RegWidth::W32), mem),
            _ => format!("lea {}, {}\n", rax, mem),
        };
        if let Loc::PhysReg(rr) = dst
            && let Some(r) = rr.as_gpr()
            && *r == X86RegGpr::RAX
        {
            return ret;
        }
        ret.push_str(&format!("mov {}, {}\n", self.t_loc_at(dst, w), rax));
        ret
    }

    fn emit_store(
        &self,
        src: &Operand<X86RegGpr, X86RegFpr>,
        addr: &Addr<X86RegGpr>,
        ty: &Type,
    ) -> String {
        let size = Self::type_size_suffix(ty);
        let w = match ty {
            Type::Bool | Type::Char => RegWidth::W8,
            Type::int | Type::float => RegWidth::W32,
            _ => RegWidth::W64,
        };
        let mem = self.mem_ref_sized(addr, size);

        format!("mov {}, {}\n", mem, self.t_operand_at(src, w))
    }

    fn emit_call(
        &self,
        dst: &Option<Loc<X86RegGpr, X86RegFpr>>,
        func: &CallTarget<X86RegGpr>,
        args: &[Operand<X86RegGpr, X86RegFpr>],
    ) -> String {
        let arg_regs = self.target_args.arg_regs();
        let fp_regs = self.target_args.fp_arg_regs();
        let mut gp_args = 0;
        let mut fp_args = 0;
        let mut out = String::new();
        for arg in args.iter() {
            let is_fp = match arg {
                Operand::Loc(loc) => match loc {
                    Loc::PhysReg(rr) => rr.is_fpr(),
                    Loc::Stack(_) => false,
                },
                Operand::ImmI64(_) => false,
                Operand::ImmF64(_) => true,
                _ => false,
            };
            if is_fp {
                out.push_str(&format!(
                    "mov {}, {}\n",
                    self.target_args.float128(fp_regs[fp_args]),
                    self.t_operand(arg)
                ));
                fp_args += 1;
            } else {
                let arg_w = Self::operand_width(arg);
                out.push_str(&format!(
                    "mov {}, {}\n",
                    self.target_args.reg_by_width(arg_regs[gp_args], arg_w),
                    self.t_operand_at(arg, arg_w)
                ));
                gp_args += 1;
            }
        }
        let target = match func {
            CallTarget::Direct(sym) => format!("__q_f_{}", sym),
            CallTarget::Indirect(reg) => self.target_args.reg64(*reg).to_string(),
        };
        out.push_str(&format!("call {}\n", target));
        if let Some(d) = dst {
            let dst_w = Self::loc_width(d);
            let rax = self.target_args.reg_by_width(X86RegGpr::RAX, dst_w);
            out.push_str(&format!("\nmov {}, {}\n", self.t_loc_at(d, dst_w), rax));
        }
        out
    }
}
