use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnumType {
    pub default_index: Option<usize>,
    pub values: Vec<LiteralValue>,
    pub variants: Option<Vec<Box<Schema>>>,
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
        I: IntoIterator<Item = Schema>,
    {
        let variants: Vec<Box<Schema>> = variants.into_iter().map(Box::new).collect();
        let mut values = vec![];

        for variant in &variants {
            if let SchemaType::Literal(lit) = &(*variant).ty {
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
