use crate::schema::SchemaField;
use crate::SchemaType;

#[derive(Clone, Debug, Default)]
pub enum UnionOperator {
    #[default]
    AnyOf,
    OneOf,
}

#[derive(Clone, Debug, Default)]
pub struct UnionType {
    pub default_index: Option<usize>,
    pub partial: bool,
    pub operator: UnionOperator,
    pub variants: Option<Vec<SchemaField>>,
    pub variants_types: Vec<Box<SchemaType>>,
}

impl UnionType {
    pub fn is_nullable(&self) -> bool {
        self.variants_types.iter().any(|v| v.is_null())
    }
}
