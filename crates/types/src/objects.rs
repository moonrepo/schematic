use crate::*;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ObjectType {
    pub key_type: Box<Schema>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub min_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub required: Option<Vec<String>>,

    pub value_type: Box<Schema>,
}

impl ObjectType {
    /// Create an indexed/mapable object schema with the provided key and value types.
    pub fn new(key_type: impl Into<Schema>, value_type: impl Into<Schema>) -> Self {
        ObjectType {
            key_type: Box::new(key_type.into()),
            value_type: Box::new(value_type.into()),
            ..ObjectType::default()
        }
    }
}

impl<K: Schematic, V: Schematic> Schematic for BTreeMap<K, V> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()))
    }
}

impl<K: Schematic, V: Schematic, S> Schematic for HashMap<K, V, S> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()))
    }
}
