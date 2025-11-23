use crate::config::error::{ConfigError, HandlerError};
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct TomlFormat;

impl<T: DeserializeOwned> SourceFormat<T> for TomlFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source.get_file_ext().map_or(false, |ext| ext == "toml")
    }

    fn parse(&self, source: &Source, content: &str) -> Result<T, ConfigError> {
        let de = toml::Deserializer::parse(content).map_err(|error| {
            ConfigError::Handler(Box::new(HandlerError::new(error.to_string())))
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
