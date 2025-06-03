use crate::*;
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TupleType {
    pub items_types: Vec<Box<Schema>>,
}

impl TupleType {
    /// Create a tuple schema with the provided item types.
    pub fn new<I, V>(items_types: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Schema>,
    {
        TupleType {
            items_types: items_types
                .into_iter()
                .map(|inner| Box::new(inner.into()))
                .collect(),
        }
    }
}

impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({})",
            self.items_types
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

macro_rules! impl_tuple {
    ($($arg: ident),*) => {
        impl<$($arg: Schematic),*> Schematic for ($($arg,)*) {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.tuple(TupleType::new([
                    $(schema.infer::<$arg>(),)*
                ]))
            }
        }
    };
}

impl_tuple!(T0);
impl_tuple!(T0, T1);
impl_tuple!(T0, T1, T2);
impl_tuple!(T0, T1, T2, T3);
impl_tuple!(T0, T1, T2, T3, T4);
impl_tuple!(T0, T1, T2, T3, T4, T5);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
