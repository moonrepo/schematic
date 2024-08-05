use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.boolean_default()
    }
}
