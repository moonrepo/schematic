use crate::{SchemaType, Schematic};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StringType {
    pub format: Option<String>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
}

macro_rules! impl_string {
    ($type:ty) => {
        impl Schematic for $type {
            fn generate_schema() -> SchemaType {
                SchemaType::String(StringType::default())
            }
        }
    };
}

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

impl Schematic for char {
    fn generate_schema() -> SchemaType {
        SchemaType::String(StringType {
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
