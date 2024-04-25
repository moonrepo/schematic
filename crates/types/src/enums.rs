use crate::literals::LiteralValue;
use crate::schema::SchemaField;

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
}
