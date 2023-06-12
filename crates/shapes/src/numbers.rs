#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IntegerType {
    pub format: Option<String>,
    pub kind: IntegerKind,
    pub max: Option<usize>,
    pub max_exclusive: Option<usize>,
    pub min: Option<usize>,
    pub min_exclusive: Option<usize>,
    pub multiple_of: Option<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum FloatKind {
    #[default]
    F32,
    F64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FloatType {
    pub format: Option<String>,
    pub kind: FloatKind,
    pub max: Option<usize>,
    pub max_exclusive: Option<usize>,
    pub min: Option<usize>,
    pub min_exclusive: Option<usize>,
    pub multiple_of: Option<usize>,
}
