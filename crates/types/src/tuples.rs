use crate::{SchemaType, Schematic};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TupleType {
    pub items_types: Vec<Box<SchemaType>>,
}

macro_rules! impl_tuple {
    ($($arg: ident),*) => {
        impl<$($arg: Schematic),*> Schematic for ($($arg,)*) {
            fn generate_schema() -> SchemaType {
                SchemaType::tuple([
                    $($arg::generate_schema(),)*
                ])
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
