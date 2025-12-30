use crate::{
    codegen::{
        CodeGen,
        regs::{reg8, reg32},
        r#struct::fields::layout_fields,
    },
    lexer::ast::{Stmt, Type},
};

pub mod fields;

impl CodeGen {
    pub fn generate_struct(&mut self, name: &str, instances: Vec<(String, Type)>, union: bool) {
        let struct_layout = layout_fields(&instances);

        self.structures.insert(
            name.to_string(),
            (
                struct_layout,
                Stmt::StructDecl {
                    name: name.to_string(),
                    instances: instances.to_vec(),
                    union,
                },
            ),
        );
    }

    pub fn generate_stack_struct_inline(
        &mut self,
        name: &str,
        instances: Vec<(String, Type)>,
        _off: usize,
        ofst: Option<usize>,
    ) -> String {
        let mut output = String::new();
        output.push_str(&format!("; defining {}\n", name));

        // Compute field offsets with alignment
        let mut stack_offset = 0;
        let mut field_offsets = Vec::new();
        for (_, ty) in &instances {
            let mut size = ty.size();

            if let Type::Struct { .. } = ty {
                size = 8;
            }

            stack_offset += size;
            field_offsets.push(stack_offset);
            stack_offset += size;
        }
        let aligned_size = (stack_offset + 15) & !15;
        output.push_str(&format!("sub rsp, {}\n", aligned_size));
        self.stack_size += aligned_size as i32;

        // Registers for arguments
        let save_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let save_fp = ["xmm0", "xmm1", "xmm2", "xmm3", "xmm4"];
        let mut gp_index = ofst.unwrap_or(0);
        let mut fp_index = 0;

        let mut offset = 0;

        // Initialize fields
        for (i, (_fname, ty)) in instances.iter().enumerate() {
            match ty {
                Type::int => {
                    output.push_str(&format!(
                        "mov dword [rsp + {}], {}\n",
                        offset,
                        reg32(save_regs[gp_index])
                    ));
                    gp_index += 1;
                }
                Type::Char | Type::Bool => {
                    output.push_str(&format!(
                        "mov byte [rsp + {}], {}\n",
                        offset,
                        reg8(save_regs[gp_index])
                    ));
                    gp_index += 1;
                }
                Type::float => {
                    output.push_str(&format!(
                        "movss [rsp + {}], {}\n",
                        offset, save_fp[fp_index]
                    ));
                    fp_index += 1;
                }
                _ => {
                    output.push_str(&format!(
                        "mov qword [rsp + {}], {}\n",
                        offset, save_regs[gp_index]
                    ));
                    gp_index += 1;
                }
            }

            offset += field_offsets[i];
        }

        output.push_str(&format!("; end {}\n", name));
        output
    }
}
