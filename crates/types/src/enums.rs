use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnumType {
    pub default_index: Option<usize>,
    pub values: Vec<LiteralValue>,
    pub variants: Option<Vec<SchemaField>>,
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
    pub fn from_macro<I>(variants: I, default_index: Option<usize>) -> Self
    where
        I: IntoIterator<Item = SchemaField>,
    {
        let variants: Vec<SchemaField> = variants.into_iter().collect();
        let mut values = vec![];

        for variant in &variants {
            if let SchemaType::Literal(lit) = &(*variant.schema).type_of {
                values.push(lit.value.clone().unwrap());
            }
        }

        EnumType {
            default_index,
            values,
            variants: Some(variants),
        }
    }
}
