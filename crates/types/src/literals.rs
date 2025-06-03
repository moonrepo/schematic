use std::fmt;

#[derive(Clone, Debug, PartialEq)]
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
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub format: Option<String>,

    pub value: LiteralValue,
}

impl LiteralType {
    /// Create a literal schema with the provided value.
    pub fn new(value: LiteralValue) -> Self {
        LiteralType {
            format: None,
            value,
        }
    }
}

impl fmt::Display for LiteralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
