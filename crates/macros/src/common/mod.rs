mod container;
mod field;
mod field_value;
mod macros;
mod variant;

pub use container::*;
pub use field::*;
pub use field_value::*;
pub use macros::*;
pub use variant::*;

#[derive(darling::FromMeta, Default)]
#[darling(default)]
pub struct SerdeMeta {
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}
