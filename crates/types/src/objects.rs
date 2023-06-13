use crate::{SchemaType, Schematic};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default)]
pub struct ObjectType {
    pub key_type: Box<SchemaType>,
    pub max_fields: Option<usize>,
    pub min_fields: Option<usize>,
    pub name: Option<String>,
    pub required: Vec<String>,
    pub value_type: Box<SchemaType>,
}

impl<K: Schematic, V: Schematic> Schematic for BTreeMap<K, V> {
    fn generate_schema() -> SchemaType {
        SchemaType::object(K::generate_schema(), V::generate_schema())
    }
}

impl<K: Schematic, V: Schematic, S> Schematic for HashMap<K, V, S> {
    fn generate_schema() -> SchemaType {
        SchemaType::object(K::generate_schema(), V::generate_schema())
    }
}
