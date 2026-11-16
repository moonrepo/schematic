use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::DeserializeOwned;
use std::path::Path;

#[derive(Default)]
pub struct TomlFormat {}

impl<T: DeserializeOwned> SourceFormat<T> for TomlFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source.get_file_ext() == Some("toml")
    }

    fn parse(
        &self,
        source: &Source,
        content: &str,
        _cache_path: Option<&Path>,
    ) -> Result<T, ConfigError> {
        // The whole document is parsed before anything is deserialized, so a
        // syntax error is at the root, as serde_path_to_error writes it
        let de = toml::Deserializer::parse(content).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: ".".into(),
            span: error.span().map(|s| s.into()),
            message: error.message().to_owned(),
        })?;

        let result: T = serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: error.path().to_string(),
            span: error.inner().span().map(|s| s.into()),
            message: error.inner().message().to_owned(),
        })?;

        Ok(result)
    }
}
