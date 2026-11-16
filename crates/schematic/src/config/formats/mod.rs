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

/// A span from one location to another, each a 1-based line and column.
/// RON's span of an error begins where the previous token ended, so any
/// whitespace it begins with is skipped, to label the token it's about,
/// or the position after it, like `123` in `(setting: 123)`.
#[cfg(feature = "ron")]
pub(crate) fn create_span_between(
    content: &str,
    start: (usize, usize),
    end: (usize, usize),
) -> SourceSpan {
    let start = SourceOffset::from_location(content, start.0, start.1).offset();
    let end = SourceOffset::from_location(content, end.0, end.1).offset();

    let start = start
        + content
            .get(start..end)
            .map(|text| text.len() - text.trim_start().len())
            .unwrap_or_default();

    (start, end.saturating_sub(start)).into()
}
