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

fn get_config_attrs(args: &ConfigArgs) -> Vec<proc_macro2::TokenStream> {
    let serde_meta = args.get_serde_meta();
    let attrs = vec![quote! { #[serde(#serde_meta) ]}];

    #[cfg(feature = "json_schema")]
    {
        attrs.push(quote! { #[derive(schemars::JsonSchema)] });
    }

    #[cfg(feature = "typescript")]
    {
        attrs.push(quote! { #[derive(ts_rs::TS)] });
    }

    attrs
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
    let partial_struct_name = format_ident!("Partial{}", struct_name);
    let struct_fields = fields.named.iter().map(Setting::from).collect::<Vec<_>>();
    let field_names = struct_fields.iter().map(|f| &f.name).collect::<Vec<_>>();

    // Attributes
    let struct_attrs = get_config_attrs(&args);

    // Config implementation

    let from_values = struct_fields
        .iter()
        .map(|f| {
            let name = &f.name;

            if f.is_nested() {
                let struct_name = f.get_nested_struct_name();

                quote! { #struct_name::from_partial(partial.#name.unwrap_or_default()) }
            } else if f.is_optional() {
                quote! { partial.#name }
            } else {
                quote! { partial.#name.unwrap_or_default() }
            }
        })
        .collect::<Vec<_>>();

    // Partial implementation

    let default_values = struct_fields
        .iter()
        .map(|f| f.get_default_value())
        .collect::<Vec<_>>();

    let merge_values = struct_fields
        .iter()
        .map(|f| {
            let name = &f.name;

            quote! {
                if next.#name.is_some() {
                    self.#name = next.#name;
                }
            }
        })
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

            fn merge(&mut self, next: Self) {
                #(#merge_values)*
            }
        }

        #[automatically_derived]
        impl schematic::Config for #struct_name {
            type Partial = #partial_struct_name;

            fn from_partial(partial: Self::Partial) -> Self {
                Self {
                    #(#field_names: #from_values),*
                }
            }
        }
    }
    .into()
}
