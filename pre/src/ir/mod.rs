use crate::lexer::ast::{Stmt, Type};

pub mod ast;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

#[derive(Debug, Clone)]
pub enum Value {
    Reg(VReg),
    Const(i64),
    Ident(String),
}

#[derive(Clone, Debug)]
pub enum IRInstruction {
    Add {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Sub {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Mul {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Div {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Mod {
        reg: VReg,
        left: Value,
        right: Value,
    },

    Eq {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Ne {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Lt {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Le {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Ge {
        reg: VReg,
        left: Value,
        right: Value,
    },
    Gt {
        reg: VReg,
        left: Value,
        right: Value,
    },

    Load {
        reg: VReg,
        addr: Value,
        offset: i32,
    },
    Store {
        value: Value,
        addr: Value,
        offset: i32,
    },

    StackAlloc {
        reg: VReg,
        size: usize,
    }, // returns pointer

    Call {
        reg: Option<VReg>,
        func: String,
        args: Vec<Value>,
    },

    Move {
        dest: VReg,
        from: Value,
    },
    AddressOf {
        dest: VReg,
        from: String,
    },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BlockId(usize);

#[derive(Debug, Clone)]
pub struct IRBlock {
    pub id: BlockId,
    pub instructions: Vec<IRInstruction>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Return {
        value: Option<Value>,
    },
    Jump {
        block: BlockId,
    },
    Branch {
        condition: Value,
        if_true: BlockId,
        if_false: BlockId,
    },
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<VReg>,
    pub ret_type: Type,
    pub blocks: Vec<IRBlock>,
    pub entry: BlockId,
}

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub functions: Vec<IRFunction>,
    pub global_consts: Vec<Stmt>,
    pub structs: Vec<StructDef>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    name: String,
    fields: Vec<(String, Type)>,
    is_union: bool,
}
