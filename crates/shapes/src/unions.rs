use crate::SchemaType;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum UnionOperator {
    AllOf,
    #[default]
    OneOf,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnionType {
    pub variant_types: Vec<Box<SchemaType>>,
    pub operator: UnionOperator,
}
