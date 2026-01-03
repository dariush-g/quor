use crate::{
    ir::{IRFunction, IRInstruction, IRProgram, VReg},
    lexer::ast::Stmt,
};

#[derive(Default)]
pub struct VRegGenerator {
    next: usize,
}

impl VRegGenerator {
    pub fn fresh(&mut self) -> VReg {
        let reg = VReg(self.next);
        self.next += 1;
        reg
    }
}

#[derive(Default)]
pub struct IRGenerator {
    vreg_gen: VRegGenerator,
    label_counter: usize,
    current_func_instrs: Vec<IRInstruction>,
}

impl IRGenerator {
    fn fresh_label(&mut self, pref: &str) -> String {
        let label = format!("{}.{}", pref, self.label_counter);
        self.label_counter += 1;
        label
    }

    pub fn generate(&mut self, program: Vec<Stmt>) -> Result<IRProgram, String> {
        let mut ir_functions = Vec::new();

        for stmt in program {
            if let Stmt::FunDecl {
                name,
                params,
                return_type,
                body,
                attributes,
            } = stmt
            {}
        }

        Err("error".to_string())
    }

    fn generate_function(&mut self, func: &Stmt) -> Result<IRFunction, String> {
        if let Stmt::FunDecl {
            name,
            params,
            return_type,
            body,
            attributes,
        } = func
        {
            let mut params = vec![];

            for _ in 0..params.len() {
                params.push(self.vreg_gen.fresh());
            }


            let ir_func = IRFunction {
                name: name.clone(),
                params,
                ret_type: return_type.clone(),
                locals: todo!(),
                blocks: todo!(),
            };
        }

        Err("error".to_string())
    }
}
