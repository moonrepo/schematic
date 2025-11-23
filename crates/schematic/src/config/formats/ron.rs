use super::create_span;
use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct RonFormat;

impl<T: DeserializeOwned> SourceFormat<T> for RonFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source.get_file_ext() == Some("ron")
    }

    fn parse(&self, source: &Source, content: &str) -> Result<T, ConfigError> {
        let de = &mut ron::Deserializer::from_str(content).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: String::new(),
            span: Some(create_span(
                content,
                error.position.line,
                error.position.col,
            )),
            message: error.to_string(),
        })?;

        let result: T = serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: error.path().to_string(),
            span: None,
            message: error.inner().to_string(),
        })?;

        Ok(result)
    }
}
