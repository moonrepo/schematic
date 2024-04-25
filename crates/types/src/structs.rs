use crate::schema::SchemaField;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructType {
    pub fields: Vec<SchemaField>,
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    /// Create a struct/shape schema with the provided fields.
    pub fn new<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = SchemaField>,
    {
        StructType {
            fields: fields.into_iter().collect(),
            ..StructType::default()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.fields.iter().all(|field| field.hidden)
    }
}
