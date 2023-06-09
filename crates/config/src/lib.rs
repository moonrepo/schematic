#![allow(clippy::result_large_err)]

mod config;
mod errors;
mod format;
mod layer;
mod loader;
mod source;
mod validator;

/// Utilities for generating TypeScript declarations.
#[cfg(feature = "typescript")]
pub mod typescript;

/// Built-in `parse_env` functions.
pub mod env;

#[doc(hidden)]
pub mod internal;

/// Built-in `merge` functions.
pub mod merge;

/// Built-in `validate` functions.
pub mod validate;

pub use config::*;
pub use errors::*;
pub use format::*;
pub use layer::*;
pub use loader::*;
pub use schematic_macros::*;
pub use source::*;
pub use starbase_styles::color;
pub use validator::*;

// We can't put these in the proc-macro crate!

#[cfg(feature = "json_schema")]
#[macro_export]
macro_rules! derive_enum {
    ($impl:item) => {
        #[derive(
            Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
        )]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}

#[cfg(not(feature = "json_schema"))]
#[macro_export]
macro_rules! derive_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}
