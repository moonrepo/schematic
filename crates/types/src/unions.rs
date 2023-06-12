use crate::{SchemaField, SchemaType};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum UnionOperator {
    AnyOf,
    #[default]
    OneOf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnionType {
    pub variant_types: Vec<Box<SchemaType>>,
    pub variants: Option<Vec<SchemaField>>,
    pub operator: UnionOperator,
}
