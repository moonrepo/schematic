use super::create_span;
use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::DeserializeOwned;
use std::path::Path;

#[derive(Default)]
pub struct RonFormat {}

impl<T: DeserializeOwned> SourceFormat<T> for RonFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source.get_file_ext() == Some("ron")
    }

    fn parse(
        &self,
        source: &Source,
        content: &str,
        _cache_path: Option<&Path>,
    ) -> Result<T, ConfigError> {
        // Every setting in a partial is an `Option`, which RON would otherwise
        // require to be written as `Some(..)`
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

        let de =
            &mut ron::Deserializer::from_str_with_options(content, &options).map_err(|error| {
                ParserError {
                    content: NamedSource::new(source.get_file_name(), content.to_owned()),
                    path: String::new(),
                    span: Some(create_span(
                        content,
                        error.span.start.line,
                        error.span.start.col,
                    )),
                    message: error.to_string(),
                }
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
