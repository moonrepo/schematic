use darling::ast::NestedMeta;
use darling::{FromAttributes, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Copy, Debug)]
pub enum SerdeIoDirection {
    From, // read / deserialize
    To,   // write / serialize
}

#[derive(Clone, Debug)]
pub enum SerdeTagFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
    // Special case for unit only enums
    Unit,
}

// #[serde(rename = "name")]
// #[serde(rename(deserialize = "de_name", serialize = "ser_name"))]
#[derive(Debug, Default, PartialEq)]
pub struct SerdeRenameArg {
    pub deserialize: Option<String>,
    pub serialize: Option<String>,
}

impl FromMeta for SerdeRenameArg {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self {
            deserialize: Some(value.into()),
            serialize: Some(value.into()),
        })
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(Default, FromMeta)]
        #[darling(default)]
        struct Rename {
            deserialize: Option<String>,
            serialize: Option<String>,
        }

        impl From<Rename> for SerdeRenameArg {
            fn from(value: Rename) -> Self {
                Self {
                    deserialize: value.deserialize,
                    serialize: value.serialize,
                }
            }
        }

        Rename::from_list(items).map(SerdeRenameArg::from)
    }
}

impl SerdeRenameArg {
    // pub fn get_name(&self, dir: SerdeIoDirection) -> Option<&str> {
    //     match dir {
    //         SerdeIoDirection::From => self.deserialize.as_deref(),
    //         SerdeIoDirection::To => self.serialize.as_deref(),
    //     }
    // }

    pub fn get_meta(&self, key: &str) -> TokenStream {
        match (self.deserialize.as_deref(), self.serialize.as_deref()) {
            (Some(de), Some(ser)) => {
                if de == ser {
                    quote! { #key = #de }
                } else {
                    quote! { #key(deserialize = #de, serialize = #ser) }
                }
            }
            (None, Some(ser)) => quote! { #key(serialize = #ser) },
            (Some(de), None) => quote! { #key(deserialize = #de) },
            _ => quote! {},
        }
    }
}

// #[serde()]
#[derive(Debug, Default, FromDeriveInput)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeContainerArgs {
    pub default: bool,
    pub deny_unknown_fields: bool,

    // struct
    pub rename: Option<SerdeRenameArg>,
    pub rename_all: Option<SerdeRenameArg>,
    pub rename_all_fields: Option<SerdeRenameArg>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[serde()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeFieldArgs {
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub default: bool,
    pub flatten: bool,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_deserializing_if: Option<String>,
    pub skip_serializing: bool,
    pub skip_serializing_if: Option<String>,

    // variant
    pub other: bool,
    pub untagged: bool,
}

// // #[config()], #[schematic()]
// #[derive(FromDeriveInput, Default)]
// #[darling(
//     default,
//     attributes(config, schematic),
//     supports(struct_named, enum_any)
// )]
// pub struct MacroArgs {
//     // config
//     pub allow_unknown_fields: bool,
//     pub context: Option<ExprPath>,
//     pub partial: PartialAttr,
//     #[cfg(feature = "env")]
//     pub env_prefix: Option<String>,

//     // serde
//     pub rename: Option<String>,
//     pub rename_all: Option<String>,
//     pub rename_all_fields: Option<String>,
//     pub serde: SerdeMeta,
// }
