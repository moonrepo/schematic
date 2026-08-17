use crate::*;
pub use indexmap::IndexMap;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EnumType {
    /// Position of the default entry. Indexes `variants` when variants are
    /// present, otherwise `values`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub default_index: Option<usize>,

    /// The literal values of this enum. When `variants` is set, this is a
    /// *derived subset* of it — variants that carry a non-literal schema
    /// (a null, say) contribute no value, so the two can differ in length.
    /// Never index this with `default_index`; use [`Self::get_default`].
    pub values: Vec<LiteralValue>,

    /// Every variant, in declaration order, keyed by name.
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

    /// Return the default value, resolving `default_index` against whichever
    /// list it indexes. Returns `None` when the default entry carries no
    /// literal value.
    pub fn get_default(&self) -> Option<&LiteralValue> {
        let index = self.default_index?;

        let Some(variants) = &self.variants else {
            return self.values.get(index);
        };

        match &variants.get_index(index)?.1.schema.ty {
            SchemaType::Literal(lit) => Some(&lit.value),
            _ => None,
        }
    }

    /// Point `default_index` at the entry holding this value. Returns false
    /// and keeps any existing default when no entry matches, as an enum can
    /// only default to a value it declares.
    pub fn set_default(&mut self, default: LiteralValue) -> bool {
        let index = match &self.variants {
            Some(variants) => variants.values().position(|variant| {
                matches!(&variant.schema.ty, SchemaType::Literal(lit) if lit.value == default)
            }),
            None => self.values.iter().position(|value| *value == default),
        };

        if index.is_none() {
            return false;
        }

        self.default_index = index;

        true
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

            let name = schema
                .name
                .take()
                .expect("Enum variant schemas require a name, as variants are keyed by it.");

            variants.insert(name, Box::new(SchemaField::new(schema)));
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
