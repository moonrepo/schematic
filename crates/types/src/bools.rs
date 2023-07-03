use crate::literals::LiteralValue;
use crate::{SchemaType, Schematic};

#[derive(Clone, Debug, Default)]
pub struct BooleanType {
    pub default: Option<LiteralValue>,
    pub name: Option<String>,
}

impl Schematic for bool {
    fn generate_schema() -> SchemaType {
        SchemaType::boolean()
    }
}
