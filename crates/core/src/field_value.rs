use syn::Type;

pub enum WrapperType {
    Arc,
    Box,
    Option,
    Rc,
}

pub struct FieldValue {
    pub original_ty: Type,
    pub wrappers: Vec<WrapperType>,
}
