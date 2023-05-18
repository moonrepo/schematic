mod config;
mod error;
mod layer;
mod loader;
mod source;
mod validator;

pub mod env;
pub mod internal;
pub mod merge;
pub mod validate;

pub use config::*;
pub use error::*;
pub use layer::*;
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
        #[serde(rename_all = "kebab-case")]
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
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}

#[cfg(all(not(feature = "json_schema"), feature = "typescript"))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, ts_rs::TS)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}

#[cfg(all(not(feature = "json_schema"), not(feature = "typescript")))]
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}
