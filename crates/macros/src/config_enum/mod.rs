mod variant;

use crate::config_enum::variant::Variant;
use crate::utils::extract_comment;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// #[config()]
#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config), supports(enum_unit))]
pub struct ConfigEnumArgs {
    // serde
    rename: Option<String>,
    rename_all: Option<String>,
}

// #[derive(ConfigEnum)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = ConfigEnumArgs::from_derive_input(&input).expect("Failed to parse arguments.");

    let Data::Enum(data) = input.data else {
        panic!("Only structs are supported.");
    };

    let enum_name = &input.ident;
    let case_format = args.rename_all.unwrap_or("kebab-case".to_owned());

    let variants = data
        .variants
        .iter()
        .map(|v| Variant::from(v, &case_format))
        .collect::<Vec<_>>();

    // let config = Config {
    //     args,
    //     comment: extract_comment(&input.attrs),
    //     name: &input.ident,
    //     settings: fields.named.iter().map(Setting::from).collect::<Vec<_>>(),
    // };

    quote! {}.into()
}
