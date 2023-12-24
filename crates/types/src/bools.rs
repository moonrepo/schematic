use crate::literals::LiteralValue;
use crate::{SchemaType, Schematic};

#[derive(Clone, Debug, Default)]
pub struct BooleanType {
    pub default: Option<LiteralValue>,
    pub name: Option<String>,
}

impl BooleanType {
    pub fn new(value: bool) -> Self {
        Self {
            default: Some(LiteralValue::Bool(value)),
            name: None,
        }
    }
}

impl Schematic for bool {
    fn generate_schema() -> SchemaType {
        SchemaType::boolean()
    }
}
