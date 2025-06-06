use crate::*;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StringType {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default: Option<LiteralValue>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub enum_values: Option<Vec<String>>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub format: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub max_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub min_length: Option<usize>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
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

impl fmt::Display for StringType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.max_length.is_some_and(|max| max == 1)
            && self.min_length.is_some_and(|min| min == 1)
        {
            write!(f, "char")
        } else if let Some(format) = &self.format {
            write!(f, "string:{format}")
        } else {
            write!(f, "string")
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
    ($type:ty, $format:expr_2021) => {
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
