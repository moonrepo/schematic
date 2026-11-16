use crate::args::NestedArg;
use crate::value::Value;
use std::ops::Deref;
use syn::{ExprPath, Type};

#[derive(Debug)]
pub struct VariantValue(Value);

impl Deref for VariantValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VariantValue {
    pub fn new(
        ty: Type,
        nested_arg: Option<&NestedArg>,
        self_ty: &Type,
        context_fn: Option<ExprPath>,
        owner: String,
    ) -> Self {
        VariantValue(Value::new(ty, nested_arg, self_ty, owner).with_context(context_fn))
    }
}
