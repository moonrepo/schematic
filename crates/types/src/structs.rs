use crate::SchemaField;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StructType {
    pub additional: bool,
    pub fields: Vec<SchemaField>,
    pub name: String,
    pub required: Vec<String>,
}
