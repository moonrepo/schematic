use crate::config::ConfigSchema;
use crate::schema::types::*;
use std::path::{Path, PathBuf};

macro_rules! schema_impl {
    ($type:ty, $instance:ident) => {
        schema_impl!($type, $instance, stringify!($type));
    };
    ($type:ty, $instance:ident, $name:expr) => {
        impl ConfigSchema for $type {
            fn generate_schema() -> Schema {
                Schema::Type {
                    name: $name.into(),
                    type_of: Type::$instance,
                    nullable: false,
                }
            }
        }
    };
}

schema_impl!((), Null, "Unit");
schema_impl!(bool, Boolean);

schema_impl!(str, String);
schema_impl!(String, String);

schema_impl!(usize, UInt);
schema_impl!(u8, UInt);
schema_impl!(u16, UInt);
schema_impl!(u32, UInt);
schema_impl!(u64, UInt);
schema_impl!(u128, UInt);

schema_impl!(isize, Int);
schema_impl!(i8, Int);
schema_impl!(i16, Int);
schema_impl!(i32, Int);
schema_impl!(i64, Int);
schema_impl!(i128, Int);

schema_impl!(f32, Float);
schema_impl!(f64, Double);

schema_impl!(Path, String);
schema_impl!(PathBuf, String);
