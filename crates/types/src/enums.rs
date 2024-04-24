use crate::literals::LiteralValue;
use crate::schema::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct EnumType {
    pub default_index: Option<usize>,
    pub values: Vec<LiteralValue>,
    pub variants: Option<Vec<SchemaField>>,
}
