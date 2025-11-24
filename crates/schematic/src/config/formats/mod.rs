#[cfg(feature = "json")]
mod json;
#[cfg(feature = "pkl")]
mod pkl;
#[cfg(feature = "ron")]
mod ron;
#[cfg(feature = "toml")]
mod toml;
#[cfg(feature = "yaml")]
mod yaml;

#[cfg(feature = "json")]
pub use json::JsonFormat;
#[cfg(feature = "pkl")]
pub use pkl::PklFormat;
#[cfg(feature = "ron")]
pub use ron::RonFormat;
#[cfg(feature = "toml")]
pub use toml::TomlFormat;
#[cfg(feature = "yaml")]
pub use yaml::YamlFormat;

use miette::{SourceOffset, SourceSpan};

pub fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}
