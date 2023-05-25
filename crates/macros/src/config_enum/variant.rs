use crate::utils::{extract_comment, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Fields, Variant as NativeVariant};

// #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(variant))]
pub struct VariantArgs {
    pub other: bool,
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

        if args.other {
            if let Fields::Unnamed(fields) = &variant.fields {
                if fields.unnamed.len() != 1 {
                    panic!("Only 1 unnamed field is supported for other.");
                }
            } else {
                panic!("Only unnamed tuple variants are supported for other.");
            }

            if args.value.is_some() {
                panic!("Value is not supported for other variants.");
            }
        } else if !matches!(variant.fields, Fields::Unit) {
            panic!("Only unit variants are supported.");
        }

        Variant {
            comment: extract_comment(&variant.attrs),
            name: &variant.ident,
            value: if args.other {
                String::new()
            } else {
                args.value
                    .clone()
                    .unwrap_or_else(|| format_case(format, variant.ident.to_string().as_str()))
            },
            args,
        }
    }

    pub fn get_display_fmt(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        if self.args.other {
            quote! {
                Self::#name(other) => f.pad(other),
            }
        } else {
            quote! {
                Self::#name => f.pad(#value),
            }
        }
    }

    pub fn get_from_str(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        if self.args.other {
            quote! {
                other => Self::#name(
                    other.try_into().map_err(|_| {
                        schematic::ConfigError::EnumInvalidOther(other.to_string())
                    })?
                ),
            }
        } else {
            quote! {
                #value => Self::#name,
            }
        }
    }

    pub fn get_unit_name(&self) -> TokenStream {
        let name = &self.name;

        if self.args.other {
            quote! {
                Self::#name(Default::default()),
            }
        } else {
            quote! {
                Self::#name,
            }
        }
    }
}
