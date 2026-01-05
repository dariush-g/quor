use crate::lexer::ast::{Expr, Type};

pub mod cfg;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

#[derive(Debug, Clone)]
pub enum Value {
    Reg(VReg),
    Const(i64),
    Local(usize),
    Global(usize),
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

    Cast {
        reg: VReg,
        src: Value,
        ty: Type,
    },

    Load {
        reg: VReg,
        addr: Value,
        offset: i32,
        ty: Type,
    },

    Store {
        value: Value,
        addr: Value,
        offset: i32,
        ty: Type,
    },

    Gep {
        dest: VReg,
        base: Value,
        index: Value, // multiplied by element size
        scale: usize,
    },

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
        src: Value, // src must be Local or Global
    },

    Declaration(AtDecl),
}

#[derive(Clone, Debug)]
pub enum AtDecl {
    Import { path: String, local: bool },
    Define { name: String, ty: Type, val: Expr },
    TrustRet,
    InlineAssembly { content: String },
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BlockId(pub usize);

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
    pub attributes: Vec<AtDecl>,
}

#[derive(Debug, Clone, Default)]
pub struct IRProgram {
    pub functions: Vec<IRFunction>,
    pub global_consts: Vec<GlobalDef>,
    pub structs: Vec<StructDef>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, Type)>,
    pub is_union: bool,
}

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub name: usize, // global id
    pub ty: Type,
    pub init: ConstInit,
}

#[derive(Debug, Clone)]
pub enum ConstInit {
    Int(i64),
    Bool(bool),
    Char(u8),
    Float32(u32),
    Null,
    AddrOfGlobal(usize),
    Bytes(Vec<u8>),
    Zeroed(usize),
}
