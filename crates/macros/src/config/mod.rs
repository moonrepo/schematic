#[allow(clippy::module_inception)]
pub mod config;
pub mod config_type;
pub mod setting;
pub mod setting_type;
pub mod variant;

use crate::config::config::{Config, ConfigArgs, SerdeArgs};
use crate::config::config_type::ConfigType;
use crate::config::setting::Setting;
use crate::config::variant::Variant;
use crate::utils::extract_common_attrs;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// #[derive(Config)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = ConfigArgs::from_derive_input(&input).expect("Failed to parse arguments.");
    let serde_args = SerdeArgs::from_derive_input(&input).unwrap_or_default();

    let config_type = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => ConfigType::NamedStruct {
                settings: fields.named.iter().map(Setting::from).collect::<Vec<_>>(),
                fields,
            },
            Fields::Unnamed(_) => {
                panic!("Unnamed structs are not supported.");
            }
            Fields::Unit => {
                panic!("Unit structs are not supported.");
            }
        },
        Data::Enum(data) => ConfigType::Enum {
            variants: data
                .variants
                .iter()
                .map(|variant| {
                    if matches!(variant.fields, Fields::Named(_)) {
                        panic!("Named enum variants are not supported.");
                    }

                    Variant::from(variant)
                })
                .collect(),
        },
        Data::Union(_) => {
            panic!("Unions are not supported.");
        }
    };

    let config = Config {
        args,
        serde_args,
        attrs: extract_common_attrs(&input.attrs),
        name: &input.ident,
        type_of: config_type,
    };

    quote! { #config }.into()
}
