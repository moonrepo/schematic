use crate::*;
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArrayType {
    pub contains: Option<bool>,
    pub items_type: Box<Schema>,
    pub max_contains: Option<usize>,
    pub max_length: Option<usize>,
    pub min_contains: Option<usize>,
    pub min_length: Option<usize>,
    pub unique: Option<bool>,
}

impl ArrayType {
    /// Create an array schema with the provided item types.
    pub fn new(items_type: impl Into<Schema>) -> Self {
        ArrayType {
            items_type: Box::new(items_type.into()),
            ..ArrayType::default()
        }
    }
}

impl<T: Schematic> Schematic for Vec<T> {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType::new(schema.infer::<T>()));
        schema.build()
    }
}

impl<T: Schematic> Schematic for &[T] {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType::new(schema.infer::<T>()));
        schema.build()
    }
}

impl<T: Schematic, const N: usize> Schematic for [T; N] {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            max_length: Some(N),
            min_length: Some(N),
            ..ArrayType::default()
        });
        schema.build()
    }
}

impl<T: Schematic, S> Schematic for HashSet<T, S> {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            unique: Some(true),
            ..ArrayType::default()
        });
        schema.build()
    }
}

impl<T: Schematic> Schematic for BTreeSet<T> {
    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            unique: Some(true),
            ..ArrayType::default()
        });
        schema.build()
    }
}
