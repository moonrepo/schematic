mod config;
mod error;
mod loader;
mod source;
mod validator;

pub mod env;
pub mod internal;
pub mod merge;

pub use config::*;
pub use error::*;
pub use loader::*;
pub use schematic_macros::*;
pub use source::*;
pub use starbase_styles::color;
pub use validator::*;

// We can't put these in the proc-macro crate!

#[cfg(all(feature = "json_schema", feature = "typescript"))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(
            Clone,
            Debug,
            Default,
            Eq,
            PartialEq,
            serde::Deserialize,
            serde::Serialize,
            schemars::JsonSchema,
            ts_rs::TS,
        )]
        $impl
    };
}

#[cfg(all(feature = "json_schema", not(feature = "typescript")))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(
            Clone,
            Debug,
            Default,
            Eq,
            PartialEq,
            serde::Deserialize,
            serde::Serialize,
            schemars::JsonSchema,
        )]
        $impl
    };
}

#[cfg(all(not(feature = "json_schema"), feature = "typescript"))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(
            Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize, ts_rs::TS,
        )]
        $impl
    };
}

#[cfg(all(not(feature = "json_schema"), not(feature = "typescript")))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        $impl
    };
}
