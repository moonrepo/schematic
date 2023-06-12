use crate::SchemaType;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TupleType {
    pub items_types: Vec<Box<SchemaType>>,
}
