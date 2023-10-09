use darling::{FromDeriveInput, FromMeta};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeArgs {
    // struct
    pub rename: Option<String>,
    pub rename_all: Option<String>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct SerdeMeta {
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}
