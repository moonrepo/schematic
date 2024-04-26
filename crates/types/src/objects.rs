use crate::*;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectType {
    pub key_type: Box<Schema>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub required: Option<Vec<String>>,
    pub value_type: Box<Schema>,
}

impl ObjectType {
    /// Create an indexed/mapable object schema with the provided key and value types.
    pub fn new(key_type: SchemaType, value_type: SchemaType) -> Self {
        ObjectType {
            key_type: Box::new(Schema::new(key_type)),
            value_type: Box::new(Schema::new(value_type)),
            ..ObjectType::default()
        }
    }
}

impl<K: Schematic, V: Schematic> Schematic for BTreeMap<K, V> {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.object(ObjectType::new(
            schema.infer_type::<K>(),
            schema.infer_type::<V>(),
        ));
        schema.build()
    }
}

impl<K: Schematic, V: Schematic, S> Schematic for HashMap<K, V, S> {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.object(ObjectType::new(
            schema.infer_type::<K>(),
            schema.infer_type::<V>(),
        ));
        schema.build()
    }
}
