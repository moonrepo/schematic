use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum LiteralType {
    Float(String),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum IntType {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl IntType {
    pub fn is_unsigned(&self) -> bool {
        match self {
            IntType::Usize
            | IntType::U8
            | IntType::U16
            | IntType::U32
            | IntType::U64
            | IntType::U128 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum Type {
    Boolean,
    Double,
    Float,
    #[default]
    Null,
    String,
    Unknown,
    Reference(String),
    Integer(IntType),
    Literal(LiteralType),
    Array(Box<Type>),
    Object(Box<Type>, Box<Type>),
    Tuple(Vec<Box<Type>>),
    Union(Vec<Box<Type>>),
    Shape(HashMap<String, SchemaField>),
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct SchemaField {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_of: Type,
    pub deprecated: bool,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
}

pub enum Schema {
    Undefined,
    // enum Foo { a, b, c }
    // type Foo = a | b | c
    Enum {
        name: String,
        variants: Vec<SchemaField>,
        fallback: Option<String>,
    },
    // struct Foo { ... }
    // interface Foo { ... }
    // type Foo = { ... }
    Shape {
        name: String,
        fields: Vec<SchemaField>,
        partial: bool,
    },
    // T
    // type Foo = T
    Type {
        name: String,
        type_of: Type,
        nullable: bool,
    },
}
