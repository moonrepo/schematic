use crate::config::ConfigSchema;
use crate::schema::types::*;
use std::path::{Path, PathBuf};

macro_rules! schema_impl {
    ($type:ty, $instance:expr) => {
        schema_impl!($type, $instance, stringify!($type));
    };
    ($type:ty, $instance:expr, $name:expr) => {
        impl ConfigSchema for $type {
            fn generate_schema() -> Schema {
                Schema::Type {
                    name: $name.into(),
                    type_of: $instance,
                    nullable: false,
                }
            }
        }
    };
}

schema_impl!((), Type::Null, "Unit");
schema_impl!(bool, Type::Boolean);

schema_impl!(str, Type::String);
schema_impl!(String, Type::String);

schema_impl!(usize, Type::Integer(IntType::Usize));
schema_impl!(u8, Type::Integer(IntType::U8));
schema_impl!(u16, Type::Integer(IntType::U16));
schema_impl!(u32, Type::Integer(IntType::U32));
schema_impl!(u64, Type::Integer(IntType::U64));
schema_impl!(u128, Type::Integer(IntType::U128));

schema_impl!(isize, Type::Integer(IntType::Isize));
schema_impl!(i8, Type::Integer(IntType::I8));
schema_impl!(i16, Type::Integer(IntType::I16));
schema_impl!(i32, Type::Integer(IntType::I32));
schema_impl!(i64, Type::Integer(IntType::I64));
schema_impl!(i128, Type::Integer(IntType::I128));

schema_impl!(f32, Type::Float);
schema_impl!(f64, Type::Double);

schema_impl!(Path, Type::String);
schema_impl!(PathBuf, Type::String);
