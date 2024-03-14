mod cacher;
mod configs;
mod errors;
mod format;
mod layer;
mod loader;
mod parser;
mod path;
mod source;
mod validator;

pub use cacher::*;
pub use configs::*;
pub use errors::*;
pub use layer::*;
pub use loader::*;
pub use parser::*;
pub use path::*;
pub use source::*;
pub use validator::*;

#[macro_export]
macro_rules! derive_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}
