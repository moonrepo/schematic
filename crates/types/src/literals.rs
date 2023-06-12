use crate::{SchemaType, Schematic};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiteralValue {
    Bool(bool),
    Float(String),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LiteralType {
    pub format: Option<String>,
    pub value: Option<LiteralValue>,
}

impl Schematic for () {
    fn generate_schema() -> SchemaType {
        SchemaType::Null
    }
}

impl Schematic for bool {
    fn generate_schema() -> SchemaType {
        SchemaType::Boolean
    }
}
