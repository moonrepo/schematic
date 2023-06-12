use crate::{SchemaField, SchemaType};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum UnionOperator {
    AnyOf,
    #[default]
    OneOf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnionType {
    pub variants: Option<Vec<SchemaField>>,
    pub variants_types: Vec<Box<SchemaType>>,
    pub operator: UnionOperator,
}
