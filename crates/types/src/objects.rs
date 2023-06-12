use crate::{SchemaType, Schematic};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectType {
    pub key_type: Box<SchemaType>,
    pub max_fields: Option<usize>,
    pub min_fields: Option<usize>,
    pub name: Option<String>,
    pub required: Vec<String>,
    pub value_type: Box<SchemaType>,
}

macro_rules! impl_map {
    ($type:ident) => {
        impl<K: Schematic, V: Schematic> Schematic for $type<K, V> {
            fn generate_schema() -> SchemaType {
                SchemaType::object(K::generate_schema(), V::generate_schema())
            }
        }
    };
}

impl_map!(BTreeMap);
impl_map!(HashMap);
