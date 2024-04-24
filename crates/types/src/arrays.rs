use crate::schema_type::SchemaType;
use crate::Schematic;
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, Default)]
pub struct ArrayType {
    pub contains: Option<bool>,
    pub items_type: Box<SchemaType>,
    pub max_contains: Option<usize>,
    pub max_length: Option<usize>,
    pub min_contains: Option<usize>,
    pub min_length: Option<usize>,
    pub unique: Option<bool>,
}

impl ArrayType {
    /// Create an array schema with the provided item types.
    pub fn new(items_type: SchemaType) -> Self {
        ArrayType {
            items_type: Box::new(items_type),
            ..ArrayType::default()
        }
    }
}

// macro_rules! impl_list {
//     ($type:ident) => {
//         impl<T: Schematic> Schematic for $type<T> {
//             fn generate_schema() -> SchemaType {
//                 SchemaType::array(T::generate_schema())
//             }
//         }
//     };
// }

// impl_list!(Vec);
// impl_list!(BTreeSet);

// impl<T: Schematic> Schematic for &[T] {
//     fn generate_schema() -> SchemaType {
//         SchemaType::array(T::generate_schema())
//     }
// }

// impl<T: Schematic, const N: usize> Schematic for [T; N] {
//     fn generate_schema() -> SchemaType {
//         SchemaType::Array(Box::new(ArrayType {
//             items_type: Box::new(T::generate_schema()),
//             max_length: Some(N),
//             min_length: Some(N),
//             ..ArrayType::default()
//         }))
//     }
// }

// impl<T: Schematic, S> Schematic for HashSet<T, S> {
//     fn generate_schema() -> SchemaType {
//         SchemaType::array(T::generate_schema())
//     }
// }
