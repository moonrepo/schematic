use crate::literals::LiteralType;
use crate::SchemaField;

#[derive(Clone, Debug, Default)]
pub struct EnumType {
    pub description: Option<String>,
    pub name: Option<String>,
    pub values: Vec<LiteralType>,
    pub variants: Option<Vec<SchemaField>>,
}
