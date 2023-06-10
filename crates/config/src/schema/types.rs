use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiteralType {
    Boolean(bool),
    Float(String),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
        matches!(
            self,
            IntType::Usize
                | IntType::U8
                | IntType::U16
                | IntType::U32
                | IntType::U64
                | IntType::U128
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Type {
    Boolean,
    Double,
    Float,
    Null,
    String,
    #[default]
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaField {
    pub name: Option<String>,
    pub description: Option<String>,
    pub kind: Type,
    pub deprecated: bool,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Schema {
    pub name: String,
    pub kind: Type,
    pub fields: Option<Vec<SchemaField>>,
    pub attributes: HashMap<String, LiteralType>,
}

pub enum Schema2 {
    Undefined,
    Builtin {
        kind: Type,
    },
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
        kind: Type,
    },
}
