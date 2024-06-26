use crate::config::errors::ConfigError;
use crate::config::parser::*;
use miette::{SourceOffset, SourceSpan};
use serde::de::DeserializeOwned;
use tracing::instrument;

pub use crate::format::Format;

fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}

impl Format {
    /// Detects a format from a provided value, either a file path or URL, by
    /// checking for a supported file extension.
    pub fn detect(value: &str) -> Result<Format, ConfigError> {
        let mut available: Vec<&str> = vec![];

        #[cfg(feature = "json")]
        {
            available.push("JSON");

            if value.ends_with(".json") {
                return Ok(Format::Json);
            }
        }

        #[cfg(feature = "toml")]
        {
            available.push("TOML");

            if value.ends_with(".toml") {
                return Ok(Format::Toml);
            }
        }

        #[cfg(feature = "yaml")]
        {
            available.push("YAML");

            if value.ends_with(".yaml") || value.ends_with(".yml") {
                return Ok(Format::Yaml);
            }
        }

        Err(ConfigError::UnsupportedFormat(
            value.to_owned(),
            available.join(", "),
        ))
    }

    /// Parse the provided content in the defined format into a partial configuration struct.
    /// On failure, will attempt to extract the path to the problematic field and source
    /// code spans (for use in `miette`).
    #[instrument(name = "parse_format", skip(content), fields(format = ?self))]
    pub fn parse<D>(&self, content: String, _location: &str) -> Result<D, ParserError>
    where
        D: DeserializeOwned,
    {
        let data: D = match self {
            Format::None => {
                unreachable!();
            }
            #[cfg(feature = "json")]
            Format::Json => {
                let content = if content.is_empty() {
                    "{}".to_owned()
                } else {
                    content
                };

                let de = &mut serde_json::Deserializer::from_str(&content);

                serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                    // content: NamedSource::new(location, content.to_owned()),
                    content: content.to_owned(),
                    path: error.path().to_string(),
                    span: Some(create_span(
                        &content,
                        error.inner().line(),
                        error.inner().column(),
                    )),
                    message: error.inner().to_string(),
                })?
            }

            #[cfg(feature = "toml")]
            Format::Toml => {
                let de = toml::Deserializer::new(&content);

                serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                    // content: NamedSource::new(location, content.to_owned()),
                    content: content.to_owned(),
                    path: error.path().to_string(),
                    span: error.inner().span().map(|s| s.into()),
                    message: error.inner().message().to_owned(),
                })?
            }

            #[cfg(feature = "yaml")]
            Format::Yaml => {
                use serde::de::IntoDeserializer;

                // First pass, convert string to value
                let de = serde_yaml::Deserializer::from_str(&content);
                let mut result: serde_yaml::Value =
                    serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                        // content: NamedSource::new(location, content.to_owned()),
                        content: content.to_owned(),
                        path: error.path().to_string(),
                        span: error
                            .inner()
                            .location()
                            .map(|s| create_span(&content, s.line(), s.column())),
                        message: error.inner().to_string(),
                    })?;

                // Applies anchors/aliases/references
                result.apply_merge().map_err(|error| ParserError {
                    // content: NamedSource::new(location, content.to_owned()),
                    content: content.to_owned(),
                    path: String::new(),
                    span: error.location().map(|s| (s.line(), s.column()).into()),
                    message: error.to_string(),
                })?;

                // Second pass, convert value to struct
                let de = result.into_deserializer();

                serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                    // content: NamedSource::new(location, content.to_owned()),
                    content: content.to_owned(),
                    path: error.path().to_string(),
                    span: error
                        .inner()
                        .location()
                        .map(|s| create_span(&content, s.line(), s.column())),
                    message: error.inner().to_string(),
                })?
            }
        };

        Ok(data)
    }
}
