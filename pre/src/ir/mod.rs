use crate::lexer::ast::Type;

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

    Label {
        name: String,
    },
    Jump {
        target: String,
    },
    JumpIf {
        condition: Value,
        target: String,
    },
    JumpIfNot {
        condition: Value,
        target: String,
    },

    Call {
        reg: Option<VReg>,
        func: String,
        args: Vec<Value>,
    },
    Return {
        value: Option<Value>,
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

#[derive(Debug, Clone)]
pub struct IRBlock {
    pub label: String,
    pub instructions: Vec<IRInstruction>,
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<VReg>,
    pub ret_type: Type,
    pub locals: Vec<(String, Type)>,
    pub blocks: Vec<IRBlock>,
}

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub functions: Vec<IRFunction>,
    pub globals: Vec<(String, Type)>,
}
