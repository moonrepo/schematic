#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralType {
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
