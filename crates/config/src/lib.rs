mod config;
pub mod env;
mod error;
pub mod internal;
mod loader;
pub mod merge;
mod source;

pub use config::*;
pub use error::*;
pub use loader::*;
pub use schematic_macros::*;
pub use source::*;

// We can't put this in a proc-macro crate!
#[macro_export]
macro_rules! config_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[cfg_attr(feature = "json_schema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
        $impl
    };
}
