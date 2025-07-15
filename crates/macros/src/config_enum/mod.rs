mod variant;

use crate::common::ContainerSerdeArgs;
use crate::config_enum::variant::Variant;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

// #[config()]
#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config), supports(enum_unit, enum_tuple))]
pub struct ConfigEnumArgs {
    before_parse: Option<String>,

    // serde
    rename: Option<String>,
    rename_all: Option<String>,
    rename_all_fields: Option<String>,
}

// #[derive(ConfigEnum)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = ConfigEnumArgs::from_derive_input(&input).expect("Failed to parse arguments.");
    let serde_args = ContainerSerdeArgs::from_derive_input(&input).unwrap_or_default();

    let Data::Enum(data) = input.data else {
        panic!("Only unit enums are supported.");
    };

    #[cfg(feature = "schema")]
    let attrs = crate::utils::extract_common_attrs(&input.attrs);

    let enum_name = &input.ident;

    #[cfg(feature = "schema")]
    let meta_name = args
        .rename
        .as_deref()
        .or(serde_args.rename.as_deref())
        .map(|n| n.to_owned())
        .unwrap_or(enum_name.to_string());

    let casing_format = args
        .rename_all_fields
        .as_deref()
        .or(args.rename_all.as_deref())
        .or(serde_args.rename_all_fields.as_deref())
        .or(serde_args.rename_all.as_deref())
        .unwrap_or("kebab-case");

    // Extract unit variants
    let variants = data
        .variants
        .iter()
        .map(|v| Variant::from(v, casing_format))
        .collect::<Vec<_>>();

    // Render variants to tokens
    let mut unit_names = vec![];
    let mut display_stmts = vec![];
    let mut from_stmts = vec![];
    let mut schema_types = vec![];
    let mut has_fallback = false;
    #[cfg(feature = "schema")]
    let mut default_index = None;

    #[cfg_attr(not(feature = "schema"), allow(unused_variables))]
    for (index, variant) in variants.into_iter().enumerate() {
        unit_names.push(variant.get_unit_name());
        display_stmts.push(variant.get_display_fmt());
        from_stmts.push(variant.get_from_str());
        schema_types.push(variant.get_schema_type());

        #[cfg(feature = "schema")]
        if variant.default {
            default_index = Some(index);
        }

        if variant.args.fallback {
            if has_fallback {
                panic!("Only 1 fallback variant is supported.")
            }

            has_fallback = true;
        }
    }

    let before_parse = if let Some(parser) = &args.before_parse {
        if parser == "lowercase" {
            quote! {
                let value = value.to_lowercase();
                let value = value.as_str();
            }
        } else if parser == "UPPERCASE" {
            quote! {
                let value = value.to_uppercase();
                let value = value.as_str();
            }
        } else {
            panic!("Unknown `before_parse` value {parser}");
        }
    } else {
        quote! {}
    };

    let from_fallback = if has_fallback {
        quote! {}
    } else {
        quote! {
            unknown => return Err(schematic::ConfigError::EnumUnknownVariant(unknown.to_owned())),
        }
    };

    let mut impls = vec![];

    impls.push(quote! {
        #[automatically_derived]
        impl schematic::ConfigEnum for #enum_name {
            fn variants() -> Vec<#enum_name> {
                vec![
                    #(#unit_names),*
                ]
            }
        }

        #[automatically_derived]
        impl std::str::FromStr for #enum_name {
            type Err = schematic::ConfigError;

            fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
                #before_parse
                Ok(match value {
                    #(#from_stmts)*
                    #from_fallback
                })
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<String> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
                std::str::FromStr::from_str(&value)
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<&String> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: &String) -> std::result::Result<Self, Self::Error> {
                std::str::FromStr::from_str(value)
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<&str> for #enum_name {
            type Error = schematic::ConfigError;

            fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
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
    });

    #[cfg(feature = "schema")]
    {
        use crate::utils::{
            extract_comment, extract_deprecated, instrument_quote, map_option_argument_quote,
        };

        let default_index = map_option_argument_quote(default_index);
        let instrument = instrument_quote();

        let deprecated = if let Some(comment) = extract_deprecated(&attrs) {
            quote! { schema.set_deprecated(#comment); }
        } else {
            quote! {}
        };
        let description = if let Some(comment) = extract_comment(&attrs) {
            quote! { schema.set_description(#comment); }
        } else {
            quote! {}
        };

        impls.push(quote! {
            #[automatically_derived]
            impl schematic::Schematic for #enum_name {
                fn schema_name() -> Option<String> {
                    Some(#meta_name.into())
                }

                #instrument
                fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
                    use schematic::schema::*;
                    #deprecated
                    #description
                    schema.enumerable(EnumType::from_fields(
                        [
                            #(#schema_types),*
                        ],
                        #default_index,
                    ))
                }
            }
        });
    }

    #[cfg(not(feature = "schema"))]
    {
        impls.push(quote! {
            #[automatically_derived]
            impl schematic::Schematic for #enum_name {}
        });
    }

    quote! {
        #(#impls)*
    }
    .into()
}
