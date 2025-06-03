use crate::schema::SchemaField;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StructType {
    pub fields: BTreeMap<String, Box<SchemaField>>,

    // The type is a partial nested config, like `PartialConfig`.
    // This doesn't mean it's been partialized.
    pub partial: bool,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
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

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct")
    }
}
