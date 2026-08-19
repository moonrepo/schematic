use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

// #[derive(Config)]
#[cfg(feature = "config")]
#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    use schematic_core::container::Container;

    let input: DeriveInput = parse_macro_input!(item);
    let output = Container::from(input);

    quote! { #output }.into()
}

// #[derive(Schematic)]
#[cfg(feature = "schema")]
#[proc_macro_derive(Schematic, attributes(schematic, schema))]
pub fn schematic(item: TokenStream) -> TokenStream {
    use schematic_core::container::Container;

    let input: DeriveInput = parse_macro_input!(item);
    let mut output = Container::from(input);
    output.schematic_only = true;

    quote! { #output }.into()
}
