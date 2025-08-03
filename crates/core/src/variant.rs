use crate::args::{SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg};
use crate::container::ContainerArgs;
use crate::utils::ImplResult;
use darling::FromAttributes;
use quote::quote;
use std::rc::Rc;
use syn::{Attribute, Fields, Ident, Variant as NativeVariant};

// #[setting()], #[schema()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting, schema))]
pub struct VariantArgs {
    pub default: bool,

    // serde
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub untagged: bool,
}

#[derive(Debug)]
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
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap();

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

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn impl_partial_default_value(&self) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.value {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|_| {
                        quote! { Default::default() }
                    })
                    .collect::<Vec<_>>();

                quote! { #name(#(#fields),*) }
            }
            Fields::Unit => quote! { #name },
        };

        res
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        ImplResult::default()
    }
}
