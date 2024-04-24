#[derive(Clone, Debug)]
pub enum LiteralValue {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(isize),
    UInt(usize),
    String(String),
}

#[derive(Clone, Debug, Default)]
pub struct LiteralType {
    pub format: Option<String>,
    pub value: Option<LiteralValue>,
}

impl LiteralType {
    /// Create a literal schema with the provided value.
    pub fn new(value: LiteralValue) -> Self {
        LiteralType {
            value: Some(value),
            ..LiteralType::default()
        }
    }
}
