mod variant;

use crate::config_enum::variant::Variant;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(
    default,
    allow_unknown_fields,
    attributes(serde),
    supports(enum_unit, enum_tuple)
)]
pub struct SerdeArgs {
    rename_all: Option<String>,
}

// #[derive(ConfigEnum)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let serde_args =
        SerdeArgs::from_derive_input(&input).expect("Failed to parse serde arguments.");

    let Data::Enum(data) = input.data else {
        panic!("Only unit enums are supported.");
    };

    let enum_name = &input.ident;
    let case_format = serde_args
        .rename_all
        .unwrap_or_else(|| "kebab-case".to_owned());

    // Extract unit variants
    let variants = data
        .variants
        .iter()
        .map(|v| Variant::from(v, &case_format))
        .collect::<Vec<_>>();

    // Render variants to tokens
    let unit_names = variants
        .iter()
        .map(|v| v.get_unit_name())
        .collect::<Vec<_>>();

    let display_stmts = variants
        .iter()
        .map(|v| v.get_display_fmt())
        .collect::<Vec<_>>();

    let from_stmts = variants
        .iter()
        .map(|v| v.get_from_str())
        .collect::<Vec<_>>();

    quote! {
        impl #enum_name {
            pub fn variants() -> Vec<#enum_name> {
                vec![
                    #(#unit_names)*
                ]
            }
        }

        #[automatically_derived]
        impl std::str::FromStr for #enum_name {
            type Err = schematic::ConfigError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    #(#from_stmts)*
                    // unknown => return Err(schematic::ConfigError::EnumUnknownVariant(unknown.to_owned())),
                })
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<String> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                std::str::FromStr::from_str(&value)
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<&String> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: &String) -> Result<Self, Self::Error> {
                std::str::FromStr::from_str(value)
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<&str> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                std::str::FromStr::from_str(value)
            }
        }

        #[automatically_derived]
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_stmts)*
                }
            }
        }
    }
    .into()
}
