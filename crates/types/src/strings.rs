use crate::*;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StringType {
    pub default: Option<LiteralValue>,
    pub enum_values: Option<Vec<String>>,
    pub format: Option<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
}

impl StringType {
    /// Create a string schema with the provided default value.
    pub fn new(value: impl AsRef<str>) -> Self {
        StringType {
            default: Some(LiteralValue::String(value.as_ref().to_owned())),
            ..StringType::default()
        }
    }
}

macro_rules! impl_string {
    ($type:ty) => {
        impl Schematic for $type {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string_default()
            }
        }
    };
}

macro_rules! impl_string_format {
    ($type:ty, $format:expr) => {
        impl Schematic for $type {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string(StringType {
                    format: Some($format.into()),
                    ..StringType::default()
                })
            }
        }
    };
}

impl Schematic for char {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.string(StringType {
            max_length: Some(1),
            min_length: Some(1),
            ..StringType::default()
        })
    }
}

impl_string!(str);
impl_string!(&str);
impl_string!(String);

impl_string_format!(Path, "path");
impl_string_format!(&Path, "path");
impl_string_format!(PathBuf, "path");

impl_string_format!(Ipv4Addr, "ipv4");
impl_string_format!(Ipv6Addr, "ipv6");

impl_string_format!(SystemTime, "time");
impl_string_format!(Duration, "duration");
