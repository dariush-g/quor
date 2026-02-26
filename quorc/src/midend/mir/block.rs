use std::collections::HashMap;

use crate::backend::lir::regalloc::RegWidth;
use crate::frontend::ast::{Expr, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VReg {
    pub id: usize,
    pub ty: VRegType,
    pub width: RegWidth,
}

pub fn type_to_reg_width(ty: &Type) -> RegWidth {
    match ty {
        Type::Bool | Type::Char => RegWidth::W8,
        Type::int | Type::float => RegWidth::W32,
        Type::Long | Type::Pointer(_) => RegWidth::W64,
        _ => RegWidth::W64,
    }
}

impl VReg {
    pub fn is_gpr(&self) -> bool {
        self.ty == VRegType::Int
    }

    pub fn is_fpr(&self) -> bool {
        self.ty == VRegType::Float
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VRegType {
    Int,
    Float,
}

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

    Memcpy {
        dst: Value, // addressable (Local/Global/Addr)
        src: Value, // addressable
        size: usize,
        align: usize,
    },

    Declaration(AtDecl), // holds things line inline assembly and imports, not function attributes or struct attributes
}

#[derive(Clone, Debug)]
pub enum AtDecl {
    Import { path: String, local: bool },
    Const { name: String, ty: Type, val: Expr },
    TrustRet,
    InlineAssembly { content: String },
    Extern { name: String },
    Variadic,
    Inline,
    NoFrame,
}

impl AtDecl {
    pub fn parse_attribute(attribute: &str) -> Option<AtDecl> {
        match attribute {
            "trust_ret" => Some(AtDecl::TrustRet),
            "variadic" => Some(AtDecl::Variadic),
            "inline" => Some(AtDecl::Inline),
            "no_frame" => Some(AtDecl::NoFrame),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
    pub offset: i32,
}

#[derive(Debug, Clone, Default)]
pub struct IRProgram {
    pub externs: Vec<String>,
    pub functions: HashMap<String, IRFunction>,
    pub global_consts: Vec<GlobalDef>,
    pub structs: HashMap<String, StructDef>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: HashMap<String, (i32, Type)>,
    pub is_union: bool,
    pub size: usize,
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
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Zeroed(usize),
    Char(char),
    Array(Vec<GlobalValue>),
    Struct(Expr),
}
