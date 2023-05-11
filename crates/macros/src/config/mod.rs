pub mod setting;

// use convert_case::{Case, Casing};
use crate::config::setting::Setting;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config))]
struct ConfigArgs {
    // serde
    rename: Option<String>,
    rename_all: Option<String>,
}

impl ConfigArgs {
    pub fn get_serde_meta(&self) -> proc_macro2::TokenStream {
        let mut meta = vec![quote! { default }, quote! { deny_unknown_fields }];

        if let Some(rename) = &self.rename {
            meta.push(quote! { rename = #rename });
        }

        let rename_all = self.rename_all.as_deref().unwrap_or("camelCase");

        meta.push(quote! { rename_all = #rename_all });

        quote! {
            #(#meta),*
        }
    }
}

// #[derive(Config)]
// #[config]
pub fn macro_impl(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let args = ConfigArgs::from_derive_input(&input).expect("Failed to parse arguments.");

    let Data::Struct(data) = input.data else {
        panic!("Only structs are supported.");
    };

    let Fields::Named(fields) = data.fields else {
        panic!("Only named field structs are supported.");
    };

    let struct_name = input.ident;
    let struct_fields = fields.named.iter().map(Setting::from).collect::<Vec<_>>();
    let partial_struct_name = format_ident!("Partial{}", struct_name);

    // Attributes
    let serde_meta = args.get_serde_meta();
    let struct_attrs = vec![
        quote! { #[serde(#serde_meta) ]},
        quote! { #[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))] },
        quote! { #[cfg_attr(feature = "typescript", derive(ts_rs::TS))] },
    ];

    // Fields
    let field_names = struct_fields.iter().map(|f| &f.name).collect::<Vec<_>>();
    let default_values = struct_fields
        .iter()
        .map(|f| f.get_default_value())
        .collect::<Vec<_>>();

    quote! {
        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #(#struct_attrs)*
        pub struct #partial_struct_name {
            #(#struct_fields)*
        }

        #[automatically_derived]
        impl schematic::PartialConfig for #partial_struct_name {
            fn default_values() -> Self {
                Self {
                    #(#field_names: #default_values),*
                }
            }
        }

        #[automatically_derived]
        impl schematic::Config for #struct_name {
            type Partial = #partial_struct_name;
        }
    }
    .into()
}
