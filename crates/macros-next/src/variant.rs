use crate::args::{SerdeContainerArgs, SerdeFieldArgs};
use crate::container::ContainerArgs;
use darling::FromAttributes;
use std::rc::Rc;
use syn::{Attribute, FieldMutability, Fields, Ident, Type, Variant as NativeVariant, Visibility};

// #[setting()], #[schema()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting, schema))]
pub struct VariantArgs {}

pub struct Variant {
    // args
    pub args: VariantArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub value: Fields,
}

impl Variant {
    pub fn new(
        variant: NativeVariant,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Variant {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap_or_default();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap_or_default();

        Variant {
            args,
            attrs: variant.attrs,
            container_args,
            ident: variant.ident,
            serde_args,
            serde_container_args,
            value: variant.fields,
        }
    }
}
