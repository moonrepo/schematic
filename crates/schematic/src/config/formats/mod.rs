#[cfg(feature = "json")]
mod json;
#[cfg(feature = "pkl")]
mod pkl;
#[cfg(feature = "ron")]
mod ron;
#[cfg(feature = "toml")]
mod toml;
#[cfg(any(feature = "yaml", feature = "yml"))]
mod yaml;

use miette::{SourceOffset, SourceSpan};

pub(super) fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}
