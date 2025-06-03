use crate::*;
pub use indexmap::IndexMap;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EnumType {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default_index: Option<usize>,

    pub values: Vec<LiteralValue>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub variants: Option<IndexMap<String, Box<SchemaField>>>,
}

impl EnumType {
    /// Create an enumerable type with the provided literal values.
    pub fn new<I>(values: I) -> Self
    where
        I: IntoIterator<Item = LiteralValue>,
    {
        EnumType {
            values: values.into_iter().collect(),
            ..EnumType::default()
        }
    }

    #[doc(hidden)]
    pub fn from_schemas<I>(schemas: I, default_index: Option<usize>) -> Self
    where
        I: IntoIterator<Item = Schema>,
    {
        let mut variants = IndexMap::default();
        let mut values = vec![];

        for mut schema in schemas.into_iter() {
            if let SchemaType::Literal(lit) = &schema.ty {
                values.push(lit.value.clone());
            }

            variants.insert(
                schema.name.take().unwrap(),
                Box::new(SchemaField::new(schema)),
            );
        }

        EnumType {
            default_index,
            values,
            variants: Some(variants),
        }
    }

    #[doc(hidden)]
    pub fn from_fields<I>(variants: I, default_index: Option<usize>) -> Self
    where
        I: IntoIterator<Item = (String, SchemaField)>,
    {
        let variants: IndexMap<String, Box<SchemaField>> = variants
            .into_iter()
            .map(|(k, v)| (k, Box::new(v)))
            .collect();
        let mut values = vec![];

        for variant in variants.values() {
            if let SchemaType::Literal(lit) = &variant.schema.ty {
                values.push(lit.value.clone());
            }
        }

        EnumType {
            default_index,
            values,
            variants: Some(variants),
        }
    }
}

impl fmt::Display for EnumType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.values
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}
