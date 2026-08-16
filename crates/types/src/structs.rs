use crate::*;
use std::fmt;
use std::ops::{Range, RangeInclusive};
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct StructType {
    /// Fields in declaration order, keyed by name.
    pub fields: IndexMap<String, Box<SchemaField>>,

    // The type is a partial nested config, like `PartialConfig`.
    // This doesn't mean it's been partialized.
    pub partial: bool,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub required: Option<Vec<String>>,
}

impl StructType {
    /// Create a struct/shape schema with the provided fields.
    pub fn new<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (String, F)>,
        F: Into<SchemaField>,
    {
        StructType {
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, Box::new(v.into())))
                .collect(),
            ..StructType::default()
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.fields.values().all(|field| field.hidden)
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "struct")
    }
}

// The following types look like scalars but serde encodes them as maps,
// so the schema has to describe the map, not the type's `Display` form.

impl Schematic for Duration {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([
            ("secs".into(), schema.infer::<u64>()),
            ("nanos".into(), schema.infer::<u32>()),
        ]))
    }
}

impl Schematic for SystemTime {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([
            ("secs_since_epoch".into(), schema.infer::<u64>()),
            ("nanos_since_epoch".into(), schema.infer::<u32>()),
        ]))
    }
}

impl<T: Schematic> Schematic for Range<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([
            ("start".into(), schema.infer::<T>()),
            ("end".into(), schema.infer::<T>()),
        ]))
    }
}

impl<T: Schematic> Schematic for RangeInclusive<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([
            ("start".into(), schema.infer::<T>()),
            ("end".into(), schema.infer::<T>()),
        ]))
    }
}
