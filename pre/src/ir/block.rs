use std::collections::HashMap;

use crate::lexer::ast::{Expr, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VReg(pub usize);

#[derive(Debug, Clone)]
pub enum Value {
    Reg(VReg),
    Const(i64),
    ConstFloat(f64),
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

    Declaration(AtDecl), // holds things line inline assembly and imports, not function attributes or struct attributes
}

#[derive(Clone, Debug)]
pub enum AtDecl {
    Import { path: String, local: bool },
    Const { name: String, ty: Type, val: Expr },
    TrustRet,
    InlineAssembly { content: String },
}

impl AtDecl {
    pub fn parse_attribute(attribute: &str) -> Option<AtDecl> {
        match attribute {
            "trust_ret" => Some(AtDecl::TrustRet),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
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
    TemporaryNone,
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
    pub functions: HashMap<String, IRFunction>,
    pub global_consts: Vec<GlobalDef>,
    pub structs: HashMap<String, StructDef>,
    // pub imports: Vec<(String, bool)>, // name, is_local
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: HashMap<String, (i32, Type)>,
    pub is_union: bool,
}

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub id: usize, // global id
    pub ty: Type,
    pub value: GlobalValue,
}

#[derive(Debug, Clone)]
pub enum GlobalValue {
    Bytes(Vec<u8>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Zeroed(usize),
    Char(char),
    Struct(Expr),
}
