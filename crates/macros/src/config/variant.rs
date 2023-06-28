use crate::utils::{
    extract_comment, extract_common_attrs, format_case, has_attr, preserve_str_literal,
};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Field, Fields, Type, Variant as NativeVariant};

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeArgs {
    pub alias: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
}

// #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(variant))]
pub struct VariantArgs {
    pub default: bool,
    pub nested: bool,

    // serde
    pub rename: Option<String>,
    pub skip: bool,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub serde_args: SerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l NativeVariant,
}

impl<'l> Variant<'l> {
    pub fn from(var: &NativeVariant) -> Variant {
        Variant {
            args: VariantArgs::from_attributes(&var.attrs).unwrap_or_default(),
            serde_args: SerdeArgs::from_attributes(&var.attrs).unwrap_or_default(),
            attrs: extract_common_attrs(&var.attrs),
            name: &var.ident,
            value: var,
        }
    }

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn generate_default_value(&self) -> TokenStream {
        let name = &self.name;

        match &self.value.fields {
            Fields::Named(_) => unreachable!(),
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
        }
    }
}
