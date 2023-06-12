use crate::SchemaType;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectType {
    pub key_type: Box<SchemaType>,
    pub max_fields: Option<usize>,
    pub min_fields: Option<usize>,
    pub required: Vec<String>,
    pub value_type: Box<SchemaType>,
}
