#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "pkl")]
pub mod pkl;
#[cfg(feature = "ron")]
pub mod ron;
#[cfg(feature = "toml")]
pub mod toml;
#[cfg(any(feature = "yaml", feature = "yml"))]
pub mod yaml;

use miette::{SourceOffset, SourceSpan};

pub(super) fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}
