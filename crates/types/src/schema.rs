use crate::schema_type::SchemaType;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema {
    pub description: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,
}

/// Represents a field within a schema struct, or a variant within a schema enum/union.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub description: Option<String>,
    pub type_of: Box<SchemaType>,
    pub type_name: Option<String>,
    pub deprecated: Option<String>,
    pub env_var: Option<String>,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl SchemaField {
    /// Create a new field from a [`SchemaType`].
    pub fn from_type(name: &str, type_of: SchemaType) -> SchemaField {
        SchemaField {
            name: name.to_owned(),
            type_of: Box::new(type_of),
            ..SchemaField::default()
        }
    }

    /// Create a new field from a [`Schema`].
    pub fn from_schema(name: &str, schema: Schema) -> SchemaField {
        SchemaField {
            name: name.to_owned(),
            description: schema.description,
            type_of: Box::new(schema.type_of),
            type_name: schema.name,
            ..SchemaField::default()
        }
    }
}
