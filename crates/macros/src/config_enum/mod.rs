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
    let serde_args = SerdeArgs::from_derive_input(&input).unwrap_or_default();

    let Data::Enum(data) = input.data else {
        panic!("Only unit enums are supported.");
    };

    let enum_name = &input.ident;
    let meta_name = enum_name.to_string();
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
    let mut unit_names = vec![];
    let mut display_stmts = vec![];
    let mut from_stmts = vec![];
    let mut schema_types = vec![];
    let mut fallback_name = None;

    for variant in variants {
        unit_names.push(variant.get_unit_name());
        display_stmts.push(variant.get_display_fmt());
        from_stmts.push(variant.get_from_str());
        schema_types.push(variant.get_schema_type());

        if variant.args.fallback {
            if fallback_name.is_some() {
                panic!("Only 1 fallback variant is supported.")
            }

            fallback_name = Some(variant.name.to_string());
        }
    }

    let from_fallback = if fallback_name.is_some() {
        quote! {}
    } else {
        quote! {
            unknown => return Err(schematic::ConfigError::EnumUnknownVariant(unknown.to_owned())),
        }
    };

    quote! {
        #[automatically_derived]
        impl schematic::Schematic for #enum_name {
            #[allow(clippy::needless_update)]
            fn generate_schema() -> schematic::SchemaType {
                use schematic::schema::*;

                let variants = vec![
                    #(#schema_types),*
                ];

                SchemaType::Union(UnionType {
                    variants_types: variants.iter().map(|v| Box::new(v.type_of.clone())).collect(),
                    variants: Some(variants),
                    ..Default::default()
                })
            }
        }

        #[automatically_derived]
        impl schematic::ConfigEnum for #enum_name {
            const META: schematic::Meta = schematic::Meta {
                name: #meta_name,
                fields: &[],
            };

            fn variants() -> Vec<#enum_name> {
                vec![
                    #(#unit_names),*
                ]
            }
        }

        #[automatically_derived]
        impl std::str::FromStr for #enum_name {
            type Err = schematic::ConfigError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    #(#from_stmts)*
                    #from_fallback
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
                write!(f, "{}", match self {
                    #(#display_stmts)*
                })
            }
        }
    }
    .into()
}
