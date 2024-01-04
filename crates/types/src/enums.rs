use crate::literals::LiteralValue;
use crate::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct EnumType {
    pub default_index: Option<usize>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub values: Vec<LiteralValue>,
    pub variants: Option<Vec<SchemaField>>,
}
