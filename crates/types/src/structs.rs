use crate::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct StructType {
    pub fields: Vec<SchemaField>,
    pub name: Option<String>,
    pub partial: bool,
    pub required: Vec<String>,
}
