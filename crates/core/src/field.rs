use crate::args::{SerdeContainerArgs, SerdeFieldArgs};
use crate::container::ContainerArgs;
use darling::FromAttributes;
use std::rc::Rc;
use syn::{Attribute, Field as NativeField, FieldMutability, Ident, Type, Visibility};

// #[schema()], #[setting()]
#[derive(Debug, FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {}

#[derive(Debug)]
pub struct Field {
    // args
    pub args: FieldArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Option<Ident>, // Named
    pub index: usize,         // Unnamed
    pub mutability: FieldMutability,
    pub ty: Type,
    pub vis: Visibility,
    // data
    // pub value_type: FieldValue<'l>,
}

impl Field {
    pub fn new(
        field: NativeField,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Field {
        let args = FieldArgs::from_attributes(&field.attrs).unwrap_or_default();
        let serde_args = SerdeFieldArgs::from_attributes(&field.attrs).unwrap_or_default();

        Field {
            args,
            attrs: field.attrs,
            container_args,
            ident: field.ident,
            index: 0,
            mutability: field.mutability,
            serde_args,
            serde_container_args,
            ty: field.ty,
            vis: field.vis,
        }
    }
}
