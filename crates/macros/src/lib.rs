#![allow(unused)]

use proc_macro::TokenStream;
use quote::quote;
use schematic_core::container::{Container, ContainerMacro};
use syn::{DeriveInput, parse_macro_input};

/// Expand a derive, or report each attribute that failed to parse as a
/// compile error at the attribute, rather than panicking.
fn derive(input: DeriveInput, macro_type: ContainerMacro) -> TokenStream {
    match Container::new(input) {
        Ok(mut output) => {
            output.macro_type = macro_type;

            quote! { #output }.into()
        }
        Err(errors) => errors.write_errors().into(),
    }
}

// #[derive(Config)]
#[cfg(feature = "config")]
#[proc_macro_derive(Config, attributes(config, serde, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    derive(parse_macro_input!(item), ContainerMacro::Config)
}

// #[derive(ConfigEnum)]
#[cfg(feature = "config")]
#[proc_macro_derive(ConfigEnum, attributes(config, serde, variant))]
pub fn config_enum(item: TokenStream) -> TokenStream {
    derive(parse_macro_input!(item), ContainerMacro::ConfigUnitEnum)
}

// #[derive(Schematic)]
#[cfg(feature = "schema")]
#[proc_macro_derive(Schematic, attributes(schema, schematic, serde))]
pub fn schematic(item: TokenStream) -> TokenStream {
    derive(parse_macro_input!(item), ContainerMacro::Schematic)
}
