use crate::{SchemaType, Schematic};

#[derive(Clone, Debug)]
pub enum LiteralValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Clone, Debug, Default)]
pub struct LiteralType {
    pub format: Option<String>,
    pub name: Option<String>,
    pub value: Option<LiteralValue>,
}

impl Schematic for () {
    fn generate_schema() -> SchemaType {
        SchemaType::Null
    }
}
