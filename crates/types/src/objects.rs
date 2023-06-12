use crate::{SchemaType, Schematic};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectType {
    pub key_type: Box<SchemaType>,
    pub max_fields: Option<usize>,
    pub min_fields: Option<usize>,
    pub required: Vec<String>,
    pub value_type: Box<SchemaType>,
}

macro_rules! impl_map {
    ($type:ident) => {
        impl<K: Schematic, V: Schematic> Schematic for $type<K, V> {
            fn generate_schema() -> SchemaType {
                SchemaType::Object(ObjectType {
                    key_type: Box::new(K::generate_schema()),
                    value_type: Box::new(V::generate_schema()),
                    ..ObjectType::default()
                })
            }
        }
    };
}

impl_map!(BTreeMap);
impl_map!(HashMap);
