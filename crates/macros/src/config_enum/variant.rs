use crate::utils::{extract_comment, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Fields, Variant as NativeVariant};

// #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(variant))]
pub struct VariantArgs {
    pub value: Option<String>,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub comment: Option<String>,
    pub name: &'l Ident,
    pub value: String,
}

impl<'l> Variant<'l> {
    pub fn from<'n>(variant: &'n NativeVariant, format: &str) -> Variant<'n> {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap_or_default();

        if !matches!(variant.fields, Fields::Unit) {
            panic!("Only unit variants are supported.");
        }

        Variant {
            comment: extract_comment(&variant.attrs),
            name: &variant.ident,
            value: args
                .value
                .clone()
                .unwrap_or_else(|| format_case(format, variant.ident.to_string().as_str())),
            args,
        }
    }

    pub fn get_display_fmt(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        quote! {
            Self::#name => f.pad(#value),
        }
    }

    pub fn get_from_str(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        quote! {
            #value => Self::#name,
        }
    }
}
