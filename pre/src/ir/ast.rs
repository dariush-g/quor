use crate::{
    ir::{BlockId, IRBlock, IRFunction, IRInstruction, IRProgram, VReg},
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
    fn generate() -> Result<IRProgram, String> {
        Err(String::from("error"))
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
                blocks: todo!(),
                entry: todo!(),
            };
        }

        Err("error".to_string())
    }
}
