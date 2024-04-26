use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TupleType {
    pub items_types: Vec<Box<Schema>>,
}

impl TupleType {
    /// Create a tuple schema with the provided item types.
    pub fn new<I>(items_types: I) -> Self
    where
        I: IntoIterator<Item = SchemaType>,
    {
        TupleType {
            items_types: items_types
                .into_iter()
                .map(|inner| Box::new(Schema::new(inner)))
                .collect(),
            ..TupleType::default()
        }
    }
}

macro_rules! impl_tuple {
    ($($arg: ident),*) => {
        impl<$($arg: Schematic),*> Schematic for ($($arg,)*) {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.tuple(TupleType::new([
                    $(schema.infer_type::<$arg>(),)*
                ]));
                schema.build()
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
