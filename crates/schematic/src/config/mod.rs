mod cacher;
mod configs;
mod error;
#[cfg(feature = "extends")]
mod extender;
mod formats;
mod layer;
mod loader;
mod parser;
mod path;
mod source;
#[cfg(feature = "validate")]
mod validator;

pub use cacher::*;
pub use configs::*;
pub use error::*;
#[cfg(feature = "extends")]
pub use extender::*;
pub use layer::*;
pub use loader::*;
pub use parser::*;
pub use path::*;
pub use source::*;
#[cfg(feature = "validate")]
pub use validator::*;

#[macro_export]
macro_rules! derive_enum {
    ($impl:item) => {
        #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
        #[serde(rename_all = "kebab-case")]
        $impl
    };
}

pub type DefaultValueResult<T> = std::result::Result<Option<T>, HandlerError>;
pub type ParseEnvResult<T> = std::result::Result<Option<T>, HandlerError>;
pub type MergeResult<T> = std::result::Result<Option<T>, HandlerError>;
