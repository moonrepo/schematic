use crate::literals::LiteralValue;
use crate::schema_type::SchemaType;
use crate::Schematic;

#[derive(Clone, Debug, Default)]
pub struct BooleanType {
    pub default: Option<LiteralValue>,
}

impl BooleanType {
    pub fn new(value: bool) -> Self {
        Self {
            default: Some(LiteralValue::Bool(value)),
        }
    }
}

impl Schematic for bool {
    fn generate_schema() -> SchemaType {
        SchemaType::boolean()
    }
}
