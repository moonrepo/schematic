use darling::ast::NestedMeta;
use darling::{FromAttributes, FromDeriveInput, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields};

#[derive(Clone, Copy)]
pub enum SerdeIoDirection {
    From, // read / de
    To,   // write / ser
}

#[derive(Clone)]
pub enum SerdeTagFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
    // Special case for unit only enums
    Unit,
}

// #[serde(rename(deserialize = "de_name", serialize = "ser_name"))]
#[derive(FromMeta)]
pub enum SerdeRenameField {
    Both(String),
    Either {
        deserialize: Option<String>,
        serialize: Option<String>,
    },
}

impl SerdeRenameField {
    pub fn get_name(&self, dir: SerdeIoDirection) -> Option<&str> {
        match self {
            Self::Both(inner) => Some(inner.as_str()),
            Self::Either {
                deserialize,
                serialize,
            } => match dir {
                SerdeIoDirection::From => deserialize.as_deref(),
                SerdeIoDirection::To => serialize.as_deref(),
            },
        }
    }
}

// #[serde()]
#[derive(Default, FromDeriveInput)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeContainerArgs {
    pub default: bool,
    pub deny_unknown_fields: bool,

    // struct
    pub rename: Option<SerdeRenameField>,
    pub rename_all: Option<SerdeRenameField>,
    pub rename_all_fields: Option<SerdeRenameField>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[serde()]
#[derive(Default, FromAttributes)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeFieldArgs {
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub default: bool,
    pub flatten: bool,
    pub rename: Option<SerdeRenameField>,
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
