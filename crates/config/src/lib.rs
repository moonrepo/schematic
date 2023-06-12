#![allow(clippy::result_large_err)]

mod config;
mod errors;
mod format;
mod layer;
mod loader;
mod source;
mod validator;

/// Built-in `parse_env` functions.
pub mod env;

#[doc(hidden)]
pub mod internal;

/// Built-in `merge` functions.
pub mod merge;

/// Utilities for generating schemas.
pub mod schema;

/// Utilities for generating TypeScript types.
#[cfg(feature = "typescript")]
pub mod typescript;

/// Built-in `validate` functions.
pub mod validate;

pub use config::*;
pub use errors::*;
pub use format::*;
pub use layer::*;
pub use loader::*;
pub use schematic_macros::*;
pub use schematic_types::{SchemaField, SchemaType, Schematic};
pub use source::*;
pub use validator::*;

/// ASCII color helpers for use within error messages.
pub use starbase_styles::color;

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
