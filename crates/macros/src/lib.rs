#![allow(unused)]

use proc_macro::TokenStream;
use quote::quote;
use schematic_core::container::{Container, ContainerMacro};
use syn::{DeriveInput, parse_macro_input};

// #[derive(Config)]
#[cfg(feature = "config")]
#[proc_macro_derive(Config, attributes(config, serde, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let output = Container::from(input);

    quote! { #output }.into()
}

// #[derive(ConfigEnum)]
#[cfg(feature = "config")]
#[proc_macro_derive(ConfigEnum, attributes(config, serde, variant))]
pub fn config_enum(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let mut output = Container::from(input);
    output.macro_type = ContainerMacro::ConfigUnitEnum;

    quote! { #output }.into()
}

// #[derive(Schematic)]
#[cfg(feature = "schema")]
#[proc_macro_derive(Schematic, attributes(schema, schematic, serde))]
pub fn schematic(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let mut output = Container::from(input);
    output.macro_type = ContainerMacro::Schematic;

    quote! { #output }.into()
}
