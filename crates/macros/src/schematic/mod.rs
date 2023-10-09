// use crate::common_schema::*;
// use darling::{FromDeriveInput, FromMeta};
// use proc_macro2::{Ident, TokenStream};
// use quote::{format_ident, quote, ToTokens};
// use syn::{Attribute, ExprPath};

// // #[schematic()]
// #[derive(FromDeriveInput, Default)]
// #[darling(default, attributes(schematic), supports(struct_named, enum_any))]
// pub struct SchematicArgs {
//     allow_unknown_fields: bool,
//     context: Option<ExprPath>,
//     env_prefix: Option<String>,
//     file: Option<String>,

//     // serde
//     rename: Option<String>,
//     rename_all: Option<String>,
//     serde: SerdeMeta,
// }
