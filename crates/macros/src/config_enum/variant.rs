use crate::utils::{extract_comment, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Expr, ExprPath, Field, Fields, Meta, Type, Variant as NativeVariant};

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
            args,
            comment: extract_comment(&variant.attrs),
            name: &variant.ident,
            value: format_case(format, variant.ident.to_string().as_str()),
        }
    }
}
