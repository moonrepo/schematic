use crate::SchemaType;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub description: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,
}

/// Represents a field within a schema struct, or a variant within a schema enum/union.
#[derive(Clone, Debug, Default)]
pub struct SchemaField {
    pub name: String,
    pub description: Option<String>,
    pub type_of: SchemaType,
    pub deprecated: Option<String>,
    pub env_var: Option<String>,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl SchemaField {
    /// Create a new field with the provided name and type.
    pub fn new(name: &str, type_of: SchemaType) -> SchemaField {
        SchemaField {
            name: name.to_owned(),
            type_of,
            ..SchemaField::default()
        }
    }
}
