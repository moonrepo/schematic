use crate::SchemaType;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArrayType {
    pub items_type: Box<SchemaType>,
    pub max_contains: Option<usize>,
    pub max_length: Option<usize>,
    pub min_contains: Option<usize>,
    pub min_length: Option<usize>,
    pub unique: bool,
}
