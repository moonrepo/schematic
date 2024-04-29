use crate::schema::Schema;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructType {
    pub fields: BTreeMap<String, Box<Schema>>,
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    /// Create a struct/shape schema with the provided fields.
    pub fn new<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = (String, Schema)>,
    {
        StructType {
            fields: fields.into_iter().map(|(k, v)| (k, Box::new(v))).collect(),
            ..StructType::default()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.fields.values().all(|field| field.hidden)
    }
}
