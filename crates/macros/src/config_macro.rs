// use convert_case::{Case, Casing};
use crate::config::setting::Setting;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config))]
struct ConfigArgs {
    json_schemas: Option<bool>,
    typescript: Option<bool>,
}

// #[derive(Config)]
// #[config]
// #[config(json_schemas = true)]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let _ = ConfigArgs::from_derive_input(&input).expect("Failed to parse arguments.");

    let Data::Struct(data) = input.data else {
        panic!("Only structs are supported.");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("Only named field structs are supported.");
    };

    let struct_name = input.ident;
    let partial_struct_name = format_ident!("Partial{}", struct_name);
    let struct_fields = fields.named.iter().map(Setting::from).collect::<Vec<_>>();

    quote! {
        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        pub struct #partial_struct_name {
            #(#struct_fields)*
        }

        #[automatically_derived]
        impl schematic::PartialConfig for #partial_struct_name {
        }

        #[automatically_derived]
        impl schematic::Config for #struct_name {
            type Partial = #partial_struct_name;
        }
    }
    .into()
}
