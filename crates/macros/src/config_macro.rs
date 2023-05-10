use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config))]
struct ConfigArgs {
    json_schemas: Option<bool>,
    typescript: Option<bool>,
}

fn convert_to_optional_fields(fields: &FieldsNamed) {
    for field in &fields.named {
        // match field {}
    }
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
    let partial_namespace =
        format_ident!("partial_{}", struct_name.to_string().to_case(Case::Snake));

    quote! {
        pub mod #partial_namespace {
            pub struct #partial_struct_name {}

            #[automatically_derived]
            impl schematic::PartialConfig for #partial_struct_name {
            }
        }

        #[automatically_derived]
        impl schematic::Config for #struct_name {
            type Partial = #partial_namespace::#partial_struct_name;
        }
    }
    .into()
}
