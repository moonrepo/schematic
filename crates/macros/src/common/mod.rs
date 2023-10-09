mod container;
mod field;
mod field_value;
mod variant;

pub use container::*;
pub use field::*;
pub use field_value::*;
pub use variant::*;

use darling::{FromAttributes, FromDeriveInput, FromMeta};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct ContainerSerdeArgs {
    // struct
    pub rename: Option<String>,
    pub rename_all: Option<String>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct FieldSerdeArgs {
    pub alias: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct SerdeMeta {
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}
