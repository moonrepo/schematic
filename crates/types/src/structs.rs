use crate::schema::Schema;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructType {
    pub fields: Vec<Box<Schema>>,
    pub partial: bool,
    pub required: Option<Vec<String>>,
}

impl StructType {
    /// Create a struct/shape schema with the provided fields.
    pub fn new<I>(fields: I) -> Self
    where
        I: IntoIterator<Item = Schema>,
    {
        StructType {
            fields: fields.into_iter().map(Box::new).collect(),
            ..StructType::default()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.fields.iter().all(|field| field.hidden)
    }
}
