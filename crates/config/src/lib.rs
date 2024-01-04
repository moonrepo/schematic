#![allow(clippy::result_large_err)]
mod format;

#[cfg(feature = "config")]
mod config;

/// Built-in `parse_env` functions.
#[cfg(feature = "config")]
pub mod env;

#[cfg(feature = "config")]
#[doc(hidden)]
pub mod internal;

/// Built-in `merge` functions.
#[cfg(feature = "config")]
pub mod merge;

/// Generate schemas to render into outputs.
#[cfg(feature = "schema")]
pub mod schema;

/// Built-in `validate` functions.
#[cfg(feature = "config")]
pub mod validate;

/// ASCII color helpers for use within error messages.
#[cfg(feature = "config")]
pub use starbase_styles::color;

#[cfg(feature = "config")]
pub use config::*;

pub use format::*;
pub use schematic_macros::*;
pub use schematic_types::{SchemaField, SchemaType, Schematic};
