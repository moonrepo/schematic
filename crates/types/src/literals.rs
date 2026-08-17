use std::fmt;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum LiteralValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(isize),
    UInt(usize),
    String(String),
}

// Hand written rather than derived so that a schema always equals itself.
// Deriving it inherits `f32`/`f64` equality, under which `NaN != NaN`, and a
// schema holding a NaN default then compares unequal to its own clone.
impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::F64(a), Self::F64(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::UInt(a), Self::UInt(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bool(inner) => inner.to_string(),
                Self::F32(inner) => inner.to_string(),
                Self::F64(inner) => inner.to_string(),
                Self::Int(inner) => inner.to_string(),
                Self::UInt(inner) => inner.to_string(),
                Self::String(inner) => format!("\"{inner}\""),
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LiteralType {
    pub value: LiteralValue,
}

impl LiteralType {
    /// Create a literal schema with the provided value.
    pub fn new(value: LiteralValue) -> Self {
        LiteralType { value }
    }
}

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
