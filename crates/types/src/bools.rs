use crate::literals::LiteralValue;
use crate::schema::Schema;
use crate::schema_builder::SchemaBuilder;
use crate::Schematic;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BooleanType {
    pub default: Option<LiteralValue>,
}

impl BooleanType {
    /// Create a boolean schema with the provided default value.
    pub fn new(value: bool) -> Self {
        BooleanType {
            default: Some(LiteralValue::Bool(value)),
        }
    }
}

impl Schematic for bool {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.boolean(BooleanType::default());
        schema.build()
    }
}
