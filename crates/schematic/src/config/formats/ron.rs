use super::create_span_between;
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

        let error_at = |error: ron::error::SpannedError, path: String| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path,
            span: Some(create_span_between(
                content,
                (error.span.start.line, error.span.start.col),
                (error.span.end.line, error.span.end.col),
            )),
            message: error.code.to_string(),
        };

        let mut de = ron::Deserializer::from_str_with_options(content, &options)
            .map_err(|error| error_at(error, ".".into()))?;

        // A data error is located where the deserializer stopped, as RON
        // itself does, since the error alone holds no position
        let result: T = serde_path_to_error::deserialize(&mut de).map_err(|error| {
            let path = error.path().to_string();

            error_at(de.span_error(error.into_inner()), path)
        })?;

        // Anything after the value is an error, as it is for `ron::from_str`
        de.end()
            .map_err(|error| error_at(de.span_error(error), ".".into()))?;

        Ok(result)
    }
}
