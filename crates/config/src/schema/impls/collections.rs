use crate::config::ConfigSchema;
use crate::schema::types::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

macro_rules! schema_val_impl {
    ($type:ident) => {
        schema_val_impl!($type, stringify!($type));
    };
    ($type:ident, $name:expr) => {
        impl<T: ConfigSchema> ConfigSchema for $type<T> {
            fn generate_schema() -> Schema {
                let inner = T::generate_schema();

                Schema {
                    name: format!("{}<{}>", $name, inner.name),
                    kind: Type::Array(Box::new(inner.kind)),
                    ..Default::default()
                }
            }
        }
    };
}

schema_val_impl!(Vec);
schema_val_impl!(BTreeSet);
schema_val_impl!(HashSet);

macro_rules! schema_keyval_impl {
    ($type:ident) => {
        schema_keyval_impl!($type, stringify!($type));
    };
    ($type:ident, $name:expr) => {
        impl<K: ConfigSchema, V: ConfigSchema> ConfigSchema for $type<K, V> {
            fn generate_schema() -> Schema {
                let key = K::generate_schema();
                let val = V::generate_schema();

                Schema {
                    name: format!("{}<{}, {}>", $name, key.name, val.name),
                    kind: Type::Object(Box::new(key.kind), Box::new(val.kind)),
                    ..Default::default()
                }
            }
        }
    };
}

schema_keyval_impl!(BTreeMap);
schema_keyval_impl!(HashMap);
