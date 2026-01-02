use crate::{
    backend::x86_64::{
        CodeGen,
        regs::{reg8, reg32},
    },
    lexer::ast::{Stmt, Type},
};

impl CodeGen {
    pub fn generate_function(
        &mut self,
        name: &str,
        params: Vec<(String, Type)>,
        body: &Vec<Stmt>,
        attributes: &[String],
    ) {
        self.locals.clear();
        self.stack_size = 0;

        let epilogue = format!(".Lret_{name}");
        let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8"];

        self.output.push_str(&format!("global {name}\n{name}:\n"));
        self.output.push_str("push rbp\nmov rbp, rsp\n");

        // Calculate total stack needed for locals + parameters
        let total_param_size: usize = params.iter().map(|(_, ty)| ty.size()).sum();
        let aligned_size = (total_param_size + 15) & !15; // align to 16 bytes
        self.stack_size = aligned_size as i32;

        self.output
            .push_str(&format!("sub rsp, {}\n", aligned_size));

        let mut offset = 0;
        for (i, (param_name, ty)) in params.iter().enumerate() {
            let size = ty.size();
            offset += size;

            // Save parameter to local stack slot
            match ty {
                Type::Char | Type::Bool => {
                    self.output.push_str(&format!(
                        "mov byte [rbp - {}], {}\n",
                        offset,
                        reg8(save_regs[i])
                    ));
                }
                Type::int => {
                    self.output.push_str(&format!(
                        "mov dword [rbp - {}], {}\n",
                        offset,
                        reg32(save_regs[i])
                    ));
                }
                _ => {
                    self.output
                        .push_str(&format!("mov qword [rbp - {}], {}\n", offset, save_regs[i]));
                }
            }

            let mut base_ty = ty;

            while let Type::Pointer(inside) = base_ty {
                base_ty = inside;
            }

            // Record local info
            let struct_info = match base_ty {
                // Type::Pointer(inner) => match &**inner {
                //     Type::Struct { name, .. } => Some(name.clone()),
                //     _ => None,
                // },
                Type::Struct { name, .. } => Some(name.clone()),
                _ => None,
            };

            self.locals.insert(
                param_name.clone(),
                (struct_info, offset.try_into().unwrap(), ty.clone()),
            );
        }

        // Check if function has @trust_ret attribute
        let _has_trust_ret = attributes.contains(&"trust_ret".to_string());

        // Generate body
        for stmt in body {
            self.handle_stmt_with_epilogue(stmt, &epilogue);
        }

        // Only add default return value if not @trust_ret

        //if !has_trust_ret {
        self.output.push_str(&format!("{epilogue}:\n"));
        self.output.push_str("mov rsp, rbp\npop rbp\nret\n");
        //}
    }
}
