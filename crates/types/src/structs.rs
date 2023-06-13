use crate::SchemaField;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StructType {
    pub additional: bool,
    pub fields: Vec<SchemaField>,
    pub name: Option<String>,
    pub partial: bool,
    pub required: Vec<String>,
}
