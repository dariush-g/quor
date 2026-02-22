use crate::{
    backend::{
        lir::{
            aarch64::{A64RegFpr, A64RegGpr, A64Regs},
            regalloc::{
                Addr, CallTarget, CmpOp, LFunction, LInst, LTerm, Loc, Operand, RegRef, TargetRegs,
            },
        },
        target::TargetEmitter,
        *,
    },
    frontend::ast::Type,
    mir::block::{GlobalDef, GlobalValue},
};

#[derive(Debug, Default)]
pub struct ARMEmitter {
    target_regs: A64Regs,
}

impl ARMEmitter {
    /// Ensure an operand is in a register. Returns (preamble_asm, register_name).
    /// If the operand is already a physical register, no preamble is needed.
    /// Otherwise, loads/moves the value into `scratch`.
    fn operand_to_reg(
        &self,
        operand: &Operand<A64RegGpr, A64RegFpr>,
        scratch: &str,
    ) -> (String, String) {
        match operand {
            Operand::Loc(loc) => self.loc_to_reg(loc, scratch),
            Operand::ImmI64(i) => {
                let setup = format!("mov {}, #{}\n", scratch, i);
                (setup, scratch.to_string())
            }
            Operand::ImmF64(f) => {
                let setup = format!("mov {}, #0x{:x}\n", scratch, f.to_bits());
                (setup, scratch.to_string())
            }
            Operand::Indirect(addr) => {
                let setup = self.load_addr_value(scratch, addr);
                (setup, scratch.to_string())
            }
        }
    }

    /// Ensure a location is in a register. Returns (preamble_asm, register_name).
    fn loc_to_reg(&self, loc: &Loc<A64RegGpr, A64RegFpr>, scratch: &str) -> (String, String) {
        match loc {
            Loc::PhysReg(reg_ref) => {
                let name = match reg_ref {
                    RegRef::GprReg(r) => self.target_regs.reg64(*r).to_string(),
                    RegRef::FprReg(r) => self.target_regs.float128(*r).to_string(),
                };
                (String::new(), name)
            }
            Loc::Stack(offset) => {
                let setup = format!("ldur {}, [x29, #-{}]\n", scratch, offset);
                (setup, scratch.to_string())
            }
        }
    }

    /// Write a value from `src_reg` into `dst` location.
    fn store_to_loc(&self, dst: &Loc<A64RegGpr, A64RegFpr>, src_reg: &str) -> String {
        match dst {
            Loc::PhysReg(reg_ref) => {
                let dst_reg = match reg_ref {
                    RegRef::GprReg(r) => self.target_regs.reg64(*r),
                    RegRef::FprReg(r) => self.target_regs.float128(*r),
                };
                if dst_reg == src_reg {
                    String::new()
                } else {
                    format!("mov {}, {}\n", dst_reg, src_reg)
                }
            }
            Loc::Stack(offset) => {
                format!("stur {}, [x29, #-{}]\n", src_reg, offset)
            }
        }
    }

    /// Emit a 3-operand ALU instruction: `op dst, a, b`
    fn emit_binop(
        &self,
        op: &str,
        dst: &Loc<A64RegGpr, A64RegFpr>,
        a: &Operand<A64RegGpr, A64RegFpr>,
        b: &Operand<A64RegGpr, A64RegFpr>,
    ) -> String {
        let mut out = String::new();
        let (setup_a, reg_a) = self.operand_to_reg(a, "x16");
        let (setup_b, reg_b) = self.operand_to_reg(b, "x17");
        out.push_str(&setup_a);
        out.push_str(&setup_b);

        let (dst_reg, is_stack) = match dst {
            Loc::PhysReg(reg_ref) => {
                let name = match reg_ref {
                    RegRef::GprReg(r) => self.target_regs.reg64(*r).to_string(),
                    RegRef::FprReg(r) => self.target_regs.float128(*r).to_string(),
                };
                (name, false)
            }
            Loc::Stack(_) => ("x16".to_string(), true),
        };

        out.push_str(&format!("{} {}, {}, {}\n", op, dst_reg, reg_a, reg_b));

        if is_stack {
            out.push_str(&self.store_to_loc(dst, "x16"));
        }

        out
    }

    /// Generate `ldr scratch, [addr]` handling global symbols with adrp+add.
    fn load_addr_value(&self, scratch: &str, addr: &Addr<A64RegGpr>) -> String {
        match addr {
            Addr::Global { sym, off } => {
                let mut out = self.load_global_addr(scratch, *sym);
                if *off != 0 {
                    out.push_str(&format!("add {}, {}, #{}\n", scratch, scratch, off));
                }
                out.push_str(&format!("ldr {}, [{}]\n", scratch, scratch));
                out
            }
            _ => {
                format!("ldr {}, {}\n", scratch, self.addr_str(addr))
            }
        }
    }

    /// Load the address of a global symbol into `dst_reg`.
    fn load_global_addr(&self, dst_reg: &str, sym: usize) -> String {
        if cfg!(target_os = "macos") {
            format!(
                "adrp {}, __q_g_{}@PAGE\nadd {}, {}, __q_g_{}@PAGEOFF\n",
                dst_reg, sym, dst_reg, dst_reg, sym
            )
        } else {
            format!(
                "adrp {}, __q_g_{}\nadd {}, {}, :lo12:__q_g_{}\n",
                dst_reg, sym, dst_reg, dst_reg, sym
            )
        }
    }

    /// Format an address for use in ldr/str (non-global).
    fn addr_str(&self, addr: &Addr<A64RegGpr>) -> String {
        match addr {
            Addr::BaseOff { base, off } => {
                let base_reg = self.target_regs.reg64(*base);
                if *off == 0 {
                    format!("[{}]", base_reg)
                } else {
                    format!("[{}, #-{}]", base_reg, off)
                }
            }
            Addr::BaseIndex {
                base,
                index,
                scale,
                off,
            } => {
                let base_reg = self.target_regs.reg64(*base);
                let index_reg = self.target_regs.reg64(*index);
                if *scale <= 1 && *off == 0 {
                    format!("[{}, {}]", base_reg, index_reg)
                } else {
                    // Compute address in x16: base + index*scale + off
                    // This is handled inline by callers when needed
                    format!(
                        "[{}, {}, lsl #{}]",
                        base_reg,
                        index_reg,
                        (*scale).trailing_zeros()
                    )
                }
            }
            Addr::Global { sym, off } => {
                // Should not be used directly - callers should use load_global_addr
                if *off == 0 {
                    format!("__q_g_{}", sym)
                } else {
                    format!("__q_g_{}+{}", sym, off)
                }
            }
        }
    }

    fn emit_load(
        &self,
        dst: &Loc<A64RegGpr, A64RegFpr>,
        addr: &Addr<A64RegGpr>,
        ty: &Type,
    ) -> String {
        let mut out = String::new();

        let is_lea = matches!(ty, Type::Pointer(_) | Type::Array(_, _));

        if is_lea {
            // Pointer/array types: compute effective address (LEA equivalent)
            match addr {
                Addr::Global { sym, off } => {
                    out.push_str(&self.load_global_addr("x16", *sym));
                    if *off != 0 {
                        out.push_str(&format!("add x16, x16, #{}\n", off));
                    }
                }
                Addr::BaseOff { base, off } => {
                    let base_reg = self.target_regs.reg64(*base);
                    if *off == 0 {
                        out.push_str(&format!("mov x16, {}\n", base_reg));
                    } else {
                        out.push_str(&format!("sub x16, {}, #{}\n", base_reg, off));
                    }
                }
                Addr::BaseIndex {
                    base,
                    index,
                    scale,
                    off,
                } => {
                    let base_reg = self.target_regs.reg64(*base);
                    let index_reg = self.target_regs.reg64(*index);
                    if *scale > 1 {
                        out.push_str(&format!(
                            "add x16, {}, {}, lsl #{}\n",
                            base_reg,
                            index_reg,
                            (*scale).trailing_zeros()
                        ));
                    } else {
                        out.push_str(&format!("add x16, {}, {}\n", base_reg, index_reg));
                    }
                    if *off != 0 {
                        out.push_str(&format!("sub x16, x16, #{}\n", off));
                    }
                }
            }
            out.push_str(&self.store_to_loc(dst, "x16"));
        } else {
            // Value types: actual memory load
            let (load_instr, use_w) = match ty {
                Type::Bool | Type::Char => ("ldrb", true),
                Type::int | Type::float => ("ldr", true),
                _ => ("ldr", false),
            };

            match addr {
                Addr::Global { sym, off } => {
                    out.push_str(&self.load_global_addr("x16", *sym));
                    if *off != 0 {
                        out.push_str(&format!("add x16, x16, #{}\n", off));
                    }
                    let tmp = if use_w { "w16" } else { "x16" };
                    out.push_str(&format!("{} {}, [x16]\n", load_instr, tmp));
                }
                _ => {
                    let addr_s = self.addr_str(addr);
                    let tmp = if use_w { "w16" } else { "x16" };
                    out.push_str(&format!("{} {}, {}\n", load_instr, tmp, addr_s));
                }
            }
            out.push_str(&self.store_to_loc(dst, "x16"));
        }

        out
    }

    fn emit_store(
        &self,
        src: &Operand<A64RegGpr, A64RegFpr>,
        addr: &Addr<A64RegGpr>,
        ty: &Type,
    ) -> String {
        let mut out = String::new();

        let (store_instr, use_w) = match ty {
            Type::Bool | Type::Char => ("strb", true),
            Type::int => ("str", true),
            _ => ("str", false),
        };

        let (setup, src_reg) = self.operand_to_reg(src, "x16");
        out.push_str(&setup);

        let sized_src = if use_w && src_reg.starts_with('x') {
            format!("w{}", &src_reg[1..])
        } else {
            src_reg
        };

        match addr {
            Addr::Global { sym, off } => {
                out.push_str(&self.load_global_addr("x17", *sym));
                if *off != 0 {
                    out.push_str(&format!("add x17, x17, #{}\n", off));
                }
                out.push_str(&format!("{} {}, [x17]\n", store_instr, sized_src));
            }
            _ => {
                let addr_s = self.addr_str(addr);
                out.push_str(&format!("{} {}, {}\n", store_instr, sized_src, addr_s));
            }
        }

        out
    }

    fn emit_call(
        &self,
        dst: &Option<Loc<A64RegGpr, A64RegFpr>>,
        func: &CallTarget<A64RegGpr>,
        args: &[Operand<A64RegGpr, A64RegFpr>],
    ) -> String {
        let mut out = String::new();
        let arg_regs = self.target_regs.arg_regs();
        let fp_regs = self.target_regs.fp_arg_regs();
        let mut gp_idx = 0;
        let mut fp_idx = 0;

        for arg in args {
            let is_fp = matches!(
                arg,
                Operand::Loc(Loc::PhysReg(RegRef::FprReg(_))) | Operand::ImmF64(_)
            );

            if is_fp {
                let reg = self.target_regs.float128(fp_regs[fp_idx]);
                let (setup, src_reg) = self.operand_to_reg(arg, "x16");
                out.push_str(&setup);
                if src_reg != reg {
                    out.push_str(&format!("fmov {}, {}\n", reg, src_reg));
                }
                fp_idx += 1;
            } else {
                let reg = self.target_regs.reg64(arg_regs[gp_idx]);
                let (setup, src_reg) = self.operand_to_reg(arg, "x16");
                out.push_str(&setup);
                if src_reg != reg {
                    out.push_str(&format!("mov {}, {}\n", reg, src_reg));
                }
                gp_idx += 1;
            }
        }

        match func {
            CallTarget::Direct(sym) => {
                out.push_str(&format!("bl __q_f_{}\n", sym));
            }
            CallTarget::Indirect(reg) => {
                out.push_str(&format!("blr {}\n", self.target_regs.reg64(*reg)));
            }
        }

        if let Some(d) = dst {
            out.push_str(&self.store_to_loc(d, "x0"));
        }

        out
    }

    fn emit_cast(
        &self,
        dst: &Loc<A64RegGpr, A64RegFpr>,
        src: &Operand<A64RegGpr, A64RegFpr>,
        ty: &Type,
    ) -> String {
        let mut out = String::new();
        let (setup, src_reg) = self.operand_to_reg(src, "x16");
        out.push_str(&setup);

        match ty {
            Type::Long | Type::Pointer(_) => {
                // Sign-extend 32-bit to 64-bit
                let w_reg = if src_reg.starts_with('x') {
                    format!("w{}", &src_reg[1..])
                } else {
                    src_reg
                };
                out.push_str(&format!("sxtw x16, {}\n", w_reg));
                out.push_str(&self.store_to_loc(dst, "x16"));
            }
            Type::Bool | Type::Char => {
                out.push_str(&format!("and x16, {}, #0xff\n", src_reg));
                out.push_str(&self.store_to_loc(dst, "x16"));
            }
            _ => {
                // Default: just move
                out.push_str(&self.store_to_loc(dst, &src_reg));
            }
        }

        out
    }
}

impl TargetEmitter for ARMEmitter {
    type Reg = A64RegGpr;
    type FpReg = A64RegFpr;

    fn t_add_global_const(&mut self, constant: GlobalDef) -> String {
        let mut out = String::new();
        let symbol = format!("__q_g_{}", constant.id);
        out.push_str(&format!("{}:\n", symbol));

        match &constant.value {
            GlobalValue::Int(v) => {
                out.push_str(&format!("    .quad {}\n", v));
            }
            GlobalValue::Float(f) => {
                out.push_str(&format!("    .quad 0x{:016x}\n", f.to_bits()));
            }
            GlobalValue::Bool(b) => {
                let val = if *b { 1 } else { 0 };
                out.push_str(&format!("    .byte {}\n", val));
            }
            GlobalValue::Char(c) => {
                out.push_str(&format!("    .byte {}\n", *c as u8));
            }
            GlobalValue::String(s) => {
                out.push_str(&format!("    .asciz {:?}\n", s));
            }
            GlobalValue::Bytes(bytes) => {
                let list = bytes
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("    .byte {}\n", list));
            }
            GlobalValue::Zeroed(size) => {
                out.push_str(&format!("    .zero {}\n", size));
            }
            GlobalValue::Array(values) => {
                for val in values {
                    match val {
                        GlobalValue::Int(v) => {
                            out.push_str(&format!("    .quad {}\n", v));
                        }
                        GlobalValue::Bool(b) => {
                            let val = if *b { 1 } else { 0 };
                            out.push_str(&format!("    .byte {}\n", val));
                        }
                        GlobalValue::Char(c) => {
                            out.push_str(&format!("    .byte {}\n", *c as u8));
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
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        let mut out = String::new();

        if func.name == "main" {
            if cfg!(target_os = "macos") {
                out.push_str(".globl _main\n_main:\n");
            } else {
                out.push_str(".globl main\nmain:\n");
            }
        } else {
            out.push_str(&format!("__q_f_{}:\n", func.name));
        }

        // Save frame pointer and link register
        out.push_str("stp x29, x30, [sp, #-16]!\n");
        out.push_str("mov x29, sp\n");

        // Allocate stack frame for locals
        if ctx.frame.frame_size > 0 {
            out.push_str(&format!("sub sp, sp, #{}\n", ctx.frame.frame_size));
        }

        out
    }

    fn t_epilogue(
        &mut self,
        ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> String {
        let mut out = String::new();
        out.push_str(&format!(".Lret_{}:\n", func.name));

        // Deallocate stack frame
        if ctx.frame.frame_size > 0 {
            out.push_str("mov sp, x29\n");
        }

        // Restore frame pointer and link register, return
        out.push_str("ldp x29, x30, [sp], #16\n");
        out.push_str("ret\n");

        out
    }

    fn t_emit_inst(
        &mut self,
        inst: &LInst<Self::Reg, Self::FpReg>,
        _ctx: &mut CodegenCtx<Self::Reg, Self::FpReg>,
    ) -> String {
        match inst {
            LInst::Add { dst, a, b } => self.emit_binop("add", dst, a, b),
            LInst::Sub { dst, a, b } => self.emit_binop("sub", dst, a, b),
            LInst::Mul { dst, a, b } => self.emit_binop("mul", dst, a, b),
            LInst::Div { dst, a, b } => self.emit_binop("sdiv", dst, a, b),
            LInst::Mod { dst, a, b } => {
                // ARM64 has no remainder instruction.
                // mod = a - (a / b) * b  =>  msub dst, quotient, b, a
                let mut out = String::new();
                let (setup_a, reg_a) = self.operand_to_reg(a, "x16");
                let (setup_b, reg_b) = self.operand_to_reg(b, "x17");
                out.push_str(&setup_a);
                out.push_str(&setup_b);

                let (dst_reg, is_stack) = match dst {
                    Loc::PhysReg(reg_ref) => {
                        let name = match reg_ref {
                            RegRef::GprReg(r) => self.target_regs.reg64(*r).to_string(),
                            RegRef::FprReg(r) => self.target_regs.float128(*r).to_string(),
                        };
                        (name, false)
                    }
                    Loc::Stack(_) => ("x16".to_string(), true),
                };

                // x16 = a / b, then msub dst = a - x16 * b
                // We need a temporary for the quotient that doesn't clobber reg_a or reg_b
                // Use dst_reg for quotient if it's not a stack, otherwise use x16
                out.push_str(&format!("sdiv {}, {}, {}\n", dst_reg, reg_a, reg_b));
                // msub dst, quotient, divisor, dividend => dst = dividend - quotient * divisor
                out.push_str(&format!(
                    "msub {}, {}, {}, {}\n",
                    dst_reg, dst_reg, reg_b, reg_a
                ));

                if is_stack {
                    out.push_str(&self.store_to_loc(dst, "x16"));
                }

                out
            }
            LInst::CmpSet { dst, op, a, b } => {
                let mut out = String::new();
                let (setup_a, reg_a) = self.operand_to_reg(a, "x16");
                let (setup_b, reg_b) = self.operand_to_reg(b, "x17");
                out.push_str(&setup_a);
                out.push_str(&setup_b);

                out.push_str(&format!("cmp {}, {}\n", reg_a, reg_b));

                let cond = match op {
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                    CmpOp::Lt => "lt",
                    CmpOp::Le => "le",
                    CmpOp::Gt => "gt",
                    CmpOp::Ge => "ge",
                };

                let (dst_reg, is_stack) = match dst {
                    Loc::PhysReg(reg_ref) => {
                        let name = match reg_ref {
                            RegRef::GprReg(r) => self.target_regs.reg64(*r).to_string(),
                            RegRef::FprReg(r) => self.target_regs.float128(*r).to_string(),
                        };
                        (name, false)
                    }
                    Loc::Stack(_) => ("x16".to_string(), true),
                };

                out.push_str(&format!("cset {}, {}\n", dst_reg, cond));

                if is_stack {
                    out.push_str(&self.store_to_loc(dst, "x16"));
                }

                out
            }
            LInst::Cast { dst, src, ty } => self.emit_cast(dst, src, ty),
            LInst::Load { dst, addr, ty } => self.emit_load(dst, addr, ty),
            LInst::Store { src, addr, ty } => self.emit_store(src, addr, ty),
            LInst::Call { dst, func, args } => self.emit_call(dst, func, args),
            LInst::Mov { dst, src } => {
                let mut out = String::new();
                let (setup, src_reg) = self.operand_to_reg(src, "x16");
                out.push_str(&setup);
                out.push_str(&self.store_to_loc(dst, &src_reg));
                out
            }
            LInst::Lea { dst, addr } => {
                let mut out = String::new();
                match addr {
                    Addr::Global { sym, off } => {
                        out.push_str(&self.load_global_addr("x16", *sym));
                        if *off != 0 {
                            out.push_str(&format!("add x16, x16, #{}\n", off));
                        }
                        out.push_str(&self.store_to_loc(dst, "x16"));
                    }
                    Addr::BaseOff { base, off } => {
                        let base_reg = self.target_regs.reg64(*base);
                        if *off == 0 {
                            out.push_str(&self.store_to_loc(dst, base_reg));
                        } else {
                            out.push_str(&format!("sub x16, {}, #{}\n", base_reg, off));
                            out.push_str(&self.store_to_loc(dst, "x16"));
                        }
                    }
                    Addr::BaseIndex {
                        base,
                        index,
                        scale,
                        off,
                    } => {
                        let base_reg = self.target_regs.reg64(*base);
                        let index_reg = self.target_regs.reg64(*index);
                        if *scale > 1 {
                            out.push_str(&format!(
                                "add x16, {}, {}, lsl #{}\n",
                                base_reg,
                                index_reg,
                                (*scale).trailing_zeros()
                            ));
                        } else {
                            out.push_str(&format!("add x16, {}, {}\n", base_reg, index_reg));
                        }
                        if *off != 0 {
                            out.push_str(&format!("sub x16, x16, #{}\n", off));
                        }
                        out.push_str(&self.store_to_loc(dst, "x16"));
                    }
                }
                out
            }
            LInst::InlineAsm { asm } => format!("{}\n", asm),
        }
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
                    let (setup, src_reg) = self.operand_to_reg(operand, "x16");
                    asm.push_str(&setup);
                    if src_reg != "x0" {
                        asm.push_str(&format!("mov x0, {}\n", src_reg));
                    }
                }
                asm.push_str(&format!("b .Lret_{}\n", ctx.func.name));
            }
            LTerm::Jump { target } => {
                asm.push_str(&format!("b .Lblock_{}_{}\n", ctx.func.name, target.0));
            }
            LTerm::Branch {
                condition,
                if_true,
                if_false,
            } => {
                let (setup, cond_reg) = self.operand_to_reg(condition, "x16");
                asm.push_str(&setup);
                asm.push_str(&format!(
                    "cbnz {}, .Lblock_{}_{}\n",
                    cond_reg, ctx.func.name, if_true.0
                ));
                asm.push_str(&format!("b .Lblock_{}_{}\n", ctx.func.name, if_false.0));
            }
        }
        asm
    }

    fn t_operand(&self, operand: &Operand<Self::Reg, Self::FpReg>) -> String {
        match operand {
            Operand::Loc(loc) => self.t_loc(loc.clone()),
            Operand::ImmI64(i) => format!("#{}", i),
            Operand::ImmF64(f) => format!("#0x{:x}", f.to_bits()),
            Operand::Indirect(addr) => self.t_addr(addr.clone()),
        }
    }

    fn t_loc(&self, loc: Loc<Self::Reg, Self::FpReg>) -> String {
        match loc {
            Loc::PhysReg(reg_ref) => match reg_ref {
                RegRef::GprReg(r) => self.target_regs.reg64(r).to_owned(),
                RegRef::FprReg(r) => self.target_regs.float128(r).to_owned(),
            },
            Loc::Stack(offset) => format!("[x29, #-{}]", offset),
        }
    }

    fn t_addr(&self, addr: Addr<Self::Reg>) -> String {
        self.addr_str(&addr)
    }

    fn generate_ctx(
        func: &LFunction<Self::Reg, Self::FpReg>,
    ) -> CodegenCtx<'_, Self::Reg, Self::FpReg> {
        // Scan all instructions to find the maximum stack offset used,
        // since func.size only tracks register spill slots but not locals.
        let mut max_off = func.size;
        for block in &func.blocks {
            for inst in &block.insts {
                let offsets = match inst {
                    LInst::Store { addr: Addr::BaseOff { off, .. }, .. } => vec![*off + 8],
                    LInst::Load { addr: Addr::BaseOff { off, .. }, .. } => vec![*off + 8],
                    LInst::Mov { dst: Loc::Stack(o), .. } => vec![*o + 8],
                    LInst::Mov { src: Operand::Loc(Loc::Stack(o)), .. } => vec![*o + 8],
                    LInst::Add { dst: Loc::Stack(o), .. }
                    | LInst::Sub { dst: Loc::Stack(o), .. }
                    | LInst::Mul { dst: Loc::Stack(o), .. }
                    | LInst::Div { dst: Loc::Stack(o), .. }
                    | LInst::Mod { dst: Loc::Stack(o), .. }
                    | LInst::CmpSet { dst: Loc::Stack(o), .. }
                    | LInst::Cast { dst: Loc::Stack(o), .. } => vec![*o + 8],
                    _ => vec![],
                };
                for o in offsets {
                    if o > max_off {
                        max_off = o;
                    }
                }
            }
        }

        let frame = FrameLayout {
            frame_size: (max_off + 15) & !15, // align up to 16
            align: 16,
        };

        CodegenCtx {
            func,
            frame,
            current_block: func.entry,
        }
    }

    fn generate_function(&mut self, func: &LFunction<Self::Reg, Self::FpReg>) -> String {
        let mut func_asm = String::new();
        let ctx = &mut Self::generate_ctx(func);
        func_asm.push_str(&self.t_prologue(ctx, func));

        // Emit entry block first, then remaining blocks
        let mut blocks = func.blocks.clone();
        if let Some(entry_pos) = blocks.iter().position(|b| b.id == func.entry) {
            let entry_block = blocks.remove(entry_pos);
            blocks.insert(0, entry_block);
        }

        for block in blocks {
            func_asm.push_str(&format!(".Lblock_{}_{}:\n", func.name, block.id.0));
            for inst in block.insts {
                func_asm.push_str(&self.t_emit_inst(&inst, ctx));
            }
            func_asm.push_str(&self.t_emit_term(&block.term, ctx));
        }

        func_asm.push_str(&self.t_epilogue(ctx, func));

        func_asm
    }

    fn t_extern(&self, ext: String) -> String {
        if cfg!(target_os = "macos") {
            return format!(".extern _{ext}");
        }
        format!(".extern {ext}")
    }
}
