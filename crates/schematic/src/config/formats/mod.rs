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

use super::error::ConfigError;
use crate::format::Format;
use miette::{SourceOffset, SourceSpan};
use serde::de::DeserializeOwned;
use std::path::Path;
use tracing::instrument;

pub(super) fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}

impl Format {
    /// Parse the provided content in the defined format into a partial configuration struct.
    /// On failure, will attempt to extract the path to the problematic field and source
    /// code spans (for use in `miette`).
    #[instrument(name = "parse_format", skip(content), fields(format = ?self))]
    pub fn parse<D>(
        &self,
        location: &str,
        content: &str,
        file_path: Option<&Path>,
    ) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        match self {
            Format::None => unreachable!(),

            #[cfg(feature = "json")]
            Format::Json => json::parse(location, content),

            #[cfg(feature = "pkl")]
            Format::Pkl => pkl::parse(location, content, file_path),

            #[cfg(feature = "ron")]
            Format::Ron => ron::parse(location, content),

            #[cfg(feature = "toml")]
            Format::Toml => toml::parse(location, content),

            #[cfg(any(feature = "yaml", feature = "yml"))]
            Format::Yaml => yaml::parse(location, content),
        }
    }
}
