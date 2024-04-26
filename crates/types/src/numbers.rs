use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum IntegerKind {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    #[default]
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl IntegerKind {
    pub fn is_unsigned(&self) -> bool {
        matches!(
            self,
            Self::Usize | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::U128
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IntegerType {
    pub default: Option<LiteralValue>,
    pub enum_values: Option<Vec<isize>>,
    pub format: Option<String>,
    pub kind: IntegerKind,
    pub max: Option<isize>,
    pub max_exclusive: Option<isize>,
    pub min: Option<isize>,
    pub min_exclusive: Option<isize>,
    pub multiple_of: Option<isize>,
}

impl IntegerType {
    /// Create a signed integer schema with the provided default value.
    pub fn new(kind: IntegerKind, value: isize) -> Self {
        assert!(!kind.is_unsigned(), "must be signed");

        IntegerType {
            default: Some(LiteralValue::Int(value)),
            kind,
            ..IntegerType::default()
        }
    }

    /// Create an unsigned integer schema with the provided default value.
    pub fn new_unsigned(kind: IntegerKind, value: usize) -> Self {
        assert!(kind.is_unsigned(), "must be unsigned");

        IntegerType {
            default: Some(LiteralValue::UInt(value)),
            kind,
            ..IntegerType::default()
        }
    }

    /// Create an integer schema with the provided kind.
    pub fn new_kind(kind: IntegerKind) -> Self {
        IntegerType {
            kind,
            ..IntegerType::default()
        }
    }
}

macro_rules! impl_int {
    ($type:ty, $kind:expr) => {
        impl Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.integer(IntegerType::new_kind($kind));
                schema.build()
            }
        }
    };
}

impl_int!(usize, IntegerKind::Usize);
impl_int!(u8, IntegerKind::U8);
impl_int!(u16, IntegerKind::U16);
impl_int!(u32, IntegerKind::U32);
impl_int!(u64, IntegerKind::U64);
impl_int!(u128, IntegerKind::U128);

impl_int!(isize, IntegerKind::Isize);
impl_int!(i8, IntegerKind::I8);
impl_int!(i16, IntegerKind::I16);
impl_int!(i32, IntegerKind::I32);
impl_int!(i64, IntegerKind::I64);
impl_int!(i128, IntegerKind::I128);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum FloatKind {
    #[default]
    F32,
    F64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloatType {
    pub default: Option<LiteralValue>,
    pub enum_values: Option<Vec<f64>>,
    pub format: Option<String>,
    pub kind: FloatKind,
    pub max: Option<f64>,
    pub max_exclusive: Option<f64>,
    pub min: Option<f64>,
    pub min_exclusive: Option<f64>,
    pub multiple_of: Option<f64>,
    pub name: Option<String>,
}

impl FloatType {
    /// Create a 32-bit float schema with the provided default value.
    pub fn new_32(value: f32) -> Self {
        FloatType {
            default: Some(LiteralValue::F32(value)),
            kind: FloatKind::F32,
            ..FloatType::default()
        }
    }

    /// Create a 64-bit float schema with the provided default value.
    pub fn new_64(value: f64) -> Self {
        FloatType {
            default: Some(LiteralValue::F64(value)),
            kind: FloatKind::F64,
            ..FloatType::default()
        }
    }

    /// Create a float schema with the provided kind.
    pub fn new_kind(kind: FloatKind) -> Self {
        FloatType {
            kind,
            ..FloatType::default()
        }
    }
}

macro_rules! impl_float {
    ($type:ty, $kind:expr) => {
        impl Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.float(FloatType::new_kind($kind));
                schema.build()
            }
        }
    };
}

impl_float!(f32, FloatKind::F32);
impl_float!(f64, FloatKind::F64);
