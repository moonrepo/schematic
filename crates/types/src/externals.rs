#![allow(unused_imports, unused_macros)]

use crate::{SchemaType, Schematic, StringType};

macro_rules! impl_string_format {
    ($type:ty, $format:expr) => {
        impl Schematic for $type {
            fn generate_schema() -> SchemaType {
                SchemaType::String(StringType {
                    format: Some($format.into()),
                    ..StringType::default()
                })
            }
        }
    };
}

#[cfg(feature = "regex")]
impl_string_format!(regex::Regex, "regex");
