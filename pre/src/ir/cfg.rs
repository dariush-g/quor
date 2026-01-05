use crate::{
    ir::{BlockId, IRBlock, IRFunction, IRInstruction, IRProgram, StructDef, VReg},
    lexer::ast::Stmt,
};

#[derive(Default, Debug, Clone)]
pub struct BlockIdGen {
    next: usize,
}

impl BlockIdGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn fresh(&mut self) -> BlockId {
        let id = BlockId(self.next);
        self.next += 1;
        id
    }
}

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
    block_gen: BlockIdGen,
    blocks: Vec<IRBlock>,
    current_block: Vec<IRInstruction>,
}

impl IRGenerator {
    fn generate(mut stmts: Vec<Stmt>) -> Result<IRProgram, String> {
        let program = IRProgram::default();
        let mut ir_generator = IRGenerator::default();
        for (i, stmt) in stmts.clone().iter().enumerate() {
            if let Stmt::AtDecl(..) = stmt {
                ir_generator.generate_declaration(stmt)?;
                stmts.remove(i);
            };
        }
        for stmt in stmts {
            match stmt {
                Stmt::FunDecl { .. } => ir_generator.generate_function(&stmt),
                Stmt::StructDecl { .. } => ir_generator.generate_struct(&stmt),
                _ => Ok(()),
            }?
        }

        Err(String::from("error"))
    }

    fn generate_struct(&mut self, stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_declaration(&mut self, stmt: &Stmt) -> Result<(), String> {
        Ok(())
    }

    fn generate_function(&mut self, func: &Stmt) -> Result<(), String> {
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
                blocks: todo!(),
                entry: todo!(),
                attributes: todo!(),
            };
        }

        Ok(())
    }

    fn generate_block(&mut self, vec_stmt: Vec<Stmt>) {
        for stmt in vec_stmt {}
    }
}
