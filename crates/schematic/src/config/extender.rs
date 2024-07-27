use crate::derive_enum;
use schematic_types::{Schema, SchemaBuilder, Schematic, UnionType};

derive_enum!(
    /// Represents an extendable setting, either a string or a list of strings.
    #[serde(untagged)]
    pub enum ExtendsFrom {
        String(String),
        List(Vec<String>),
    }
);

impl Default for ExtendsFrom {
    fn default() -> Self {
        Self::List(vec![])
    }
}

impl Schematic for ExtendsFrom {
    fn schema_name() -> Option<String> {
        Some("ExtendsFrom".into())
    }

    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.union(UnionType::new_any([
            schema.infer::<String>(),
            schema.infer::<Vec<String>>(),
        ]))
    }
}
