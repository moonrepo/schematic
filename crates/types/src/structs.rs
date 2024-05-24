use crate::schema::SchemaField;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructType {
    pub fields: BTreeMap<String, Box<SchemaField>>,
    // The type is a partial nested config, like `PartialConfig`.
    // This doesn't mean it's been partialized.
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    /// Create a struct/shape schema with the provided fields.
    pub fn new<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (String, F)>,
        F: Into<SchemaField>,
    {
        StructType {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, Box::new(v.into())))
                .collect(),
            ..StructType::default()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.fields.values().all(|field| field.hidden)
    }
}
