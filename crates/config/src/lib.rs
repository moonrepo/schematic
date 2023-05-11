mod config;

pub use config::*;
pub use schematic_macros::*;

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
