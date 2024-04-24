use crate::schema::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct StructType {
    pub fields: Vec<SchemaField>,
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    pub fn is_hidden(&self) -> bool {
        self.fields.iter().all(|field| field.hidden)
    }
}
