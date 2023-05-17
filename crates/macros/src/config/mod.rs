#[allow(clippy::module_inception)]
pub mod config;
pub mod setting;
pub mod setting_type;

use crate::config::config::{Config, ConfigArgs};
use crate::config::setting::Setting;
use crate::utils::extract_comment;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// #[derive(Config)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = ConfigArgs::from_derive_input(&input).expect("Failed to parse arguments.");

    let Data::Struct(data) = input.data else {
        panic!("Only structs are supported.");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("Only named field structs are supported.");
    };

    let config = Config {
        args,
        comment: extract_comment(&input.attrs),
        name: &input.ident,
        settings: fields.named.iter().map(Setting::from).collect::<Vec<_>>(),
    };

    quote! { #config }.into()
}
