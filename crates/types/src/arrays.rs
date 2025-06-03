use crate::*;
use std::collections::{BTreeSet, HashSet};
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ArrayType {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub contains: Option<bool>,

    pub items_type: Box<Schema>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_contains: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub min_contains: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub min_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
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

impl fmt::Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.items_type)
    }
}

impl<T: Schematic> Schematic for Vec<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType::new(schema.infer::<T>()))
    }
}

impl<T: Schematic> Schematic for &[T] {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType::new(schema.infer::<T>()))
    }
}

impl<T: Schematic, const N: usize> Schematic for [T; N] {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            max_length: Some(N),
            min_length: Some(N),
            ..ArrayType::default()
        })
    }
}

impl<T: Schematic, S> Schematic for HashSet<T, S> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            unique: Some(true),
            ..ArrayType::default()
        })
    }
}

impl<T: Schematic> Schematic for BTreeSet<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.array(ArrayType {
            items_type: Box::new(schema.infer::<T>()),
            unique: Some(true),
            ..ArrayType::default()
        })
    }
}
