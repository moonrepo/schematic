use crate::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct StructType {
    pub description: Option<String>,
    pub fields: Vec<SchemaField>,
    pub name: Option<String>,
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    pub fn is_hidden(&self) -> bool {
        self.fields.iter().all(|field| field.hidden)
    }
}
