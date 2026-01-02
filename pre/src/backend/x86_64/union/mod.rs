use crate::{
    backend::x86_64::{
        CodeGen,
        regs::{reg8, reg32},
    },
    lexer::ast::Type,
};

impl CodeGen {
    pub fn _generate_stack_union_inline(
        &mut self,
        name: &str,
        instances: Vec<(String, Type)>,
        i: usize,
        off: usize,
    ) -> String {
        let mut size = 0;
        let mut output = String::new();

        for instance in &instances {
            size = instance.1.size().max(size);
        }

        output.push_str(&format!("; ----- Inline stack struct: {} -----\n", name));
        output.push_str(&format!("%define {}_size {}\n", name, size));

        output.push('\n');

        // Compute stack size
        let stack_offset = size;

        let aligned_size = (stack_offset + 7) & !7;
        output.push_str(&format!("sub rsp, {}\n", aligned_size));

        self.stack_size += aligned_size as i32;

        let ty = instances.get(i).unwrap().1.clone();
        match ty {
            Type::int => {
                output.push_str(&format!("mov dword [rbp - {}], {}\n", off, reg32("rdi")));
            }
            Type::Char | Type::Bool => {
                output.push_str(&format!("mov byte [rbp - {}], {}\n", off, reg8("rdi")));
            }
            Type::float => {
                output.push_str(&format!("movss [rbp - {}], {}\n", off, "xmm0"));
            }
            _ => {
                output.push_str(&format!("mov qword [rbp - {}], {}\n", off, "rdi"));
            }
        }
        output
    }
}
