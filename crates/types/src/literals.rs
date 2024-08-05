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

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LiteralType {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
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
