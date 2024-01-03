use crate::{SchemaField, SchemaType};

#[derive(Clone, Debug, Default)]
pub enum UnionOperator {
    #[default]
    AnyOf,
    OneOf,
}

#[derive(Clone, Debug, Default)]
pub struct UnionType {
    pub default_index: Option<usize>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub partial: bool,
    pub operator: UnionOperator,
    pub variants: Option<Vec<SchemaField>>,
    pub variants_types: Vec<Box<SchemaType>>,
}

impl UnionType {
    pub fn is_nullable(&self) -> bool {
        self.variants_types.iter().find(|v| v.is_null()).is_some()
    }
}
