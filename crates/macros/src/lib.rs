mod common;
mod utils;

#[cfg(feature = "config")]
mod config;
#[cfg(feature = "config")]
mod config_enum;
#[cfg(feature = "schema")]
mod schematic;

use common::Macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// #[derive(Config)]
#[cfg(feature = "config")]
#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let output = config::ConfigMacro(Macro::from(&input));

    quote! { #output }.into()
}

// #[derive(ConfigEnum)]
#[cfg(feature = "config")]
#[proc_macro_derive(ConfigEnum, attributes(config, variant))]
pub fn config_enum(item: TokenStream) -> TokenStream {
    config_enum::macro_impl(item)
}

// #[derive(Schematic)]
#[cfg(feature = "schema")]
#[proc_macro_derive(Schematic, attributes(schematic, field, variant))]
pub fn schematic(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let output = schematic::SchematicMacro(Macro::from(&input));

    quote! { #output }.into()
}
