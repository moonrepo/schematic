#![allow(clippy::result_large_err)]

mod config;

/// Built-in `parse_env` functions.
pub mod env;

#[doc(hidden)]
pub mod internal;

/// Built-in `merge` functions.
pub mod merge;

/// Generate schemas to render into outputs.
#[cfg(feature = "schema")]
pub mod schema;

/// Built-in `validate` functions.
pub mod validate;

/// ASCII color helpers for use within error messages.
pub use starbase_styles::color;

pub use config::*;
pub use schematic_macros::*;
pub use schematic_types::{SchemaField, SchemaType, Schematic};

#[macro_export]
macro_rules! derive_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}
