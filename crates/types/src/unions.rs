use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum UnionOperator {
    #[default]
    AnyOf,
    OneOf,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnionType {
    pub default_index: Option<usize>,
    pub partial: bool,
    pub operator: UnionOperator,
    pub variants: Option<Vec<SchemaField>>,
    pub variants_types: Vec<Box<Schema>>,
}

impl UnionType {
    /// Create an "any of" union schema.
    pub fn new_any<I>(variants_types: I) -> Self
    where
        I: IntoIterator<Item = SchemaType>,
    {
        UnionType {
            variants_types: variants_types
                .into_iter()
                .map(|inner| Box::new(Schema::new(inner)))
                .collect(),
            ..UnionType::default()
        }
    }

    /// Create a "one of" union schema.
    pub fn new_one<I>(variants_types: I) -> Self
    where
        I: IntoIterator<Item = SchemaType>,
    {
        UnionType {
            operator: UnionOperator::OneOf,
            variants_types: variants_types
                .into_iter()
                .map(|inner| Box::new(Schema::new(inner)))
                .collect(),
            ..UnionType::default()
        }
    }

    pub fn has_null(&self) -> bool {
        self.variants_types
            .iter()
            .any(|schema| schema.type_of.is_null())
    }
}
