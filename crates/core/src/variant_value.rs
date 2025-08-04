use crate::args::NestedArg;
use crate::value::Value;
use std::ops::Deref;
use syn::Type;

#[derive(Debug)]
pub struct VariantValue(Value);

impl Deref for VariantValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VariantValue {
    pub fn new(ty: Type, nested_arg: Option<&NestedArg>) -> Self {
        VariantValue(Value::new(ty, nested_arg))
    }
}
