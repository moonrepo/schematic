use crate::{SchemaField, SchemaType};

#[derive(Clone, Debug, Default)]
pub enum UnionOperator {
    AnyOf,
    #[default]
    OneOf,
}

#[derive(Clone, Debug, Default)]
pub struct UnionType {
    pub name: Option<String>,
    pub operator: UnionOperator,
    pub variants: Option<Vec<SchemaField>>,
    pub variants_types: Vec<Box<SchemaType>>,
}
