use super::create_span;
use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::DeserializeOwned;

pub fn parse<D>(name: &str, content: &str) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    let de =
        &mut serde_json::Deserializer::from_str(if content.is_empty() { "{}" } else { content });

    let result: D = serde_path_to_error::deserialize(de).map_err(|error| ParserError {
        content: NamedSource::new(name, content.to_owned()),
        path: error.path().to_string(),
        span: Some(create_span(
            content,
            error.inner().line(),
            error.inner().column(),
        )),
        message: error.inner().to_string(),
    })?;

    Ok(result)
}

#[derive(Default)]
pub struct JsonFormat;

impl SourceFormat for JsonFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source
            .get_file_ext()
            .map_or(false, |ext| ext == "json" || ext == "jsonc")
    }

    fn parse<D: DeserializeOwned>(&self, source: &Source, content: &str) -> Result<D, ConfigError> {
        let mut content = String::from(if content.is_empty() { "{}" } else { content });

        json_strip_comments::strip(&mut content).map_err(|error| {
            ConfigError::JsonStripCommentsFailed {
                file: source.get_file_name().to_owned(),
                error: Box::new(error),
            }
        })?;

        let de = &mut serde_json::Deserializer::from_str(&content);

        let result: D = serde_path_to_error::deserialize(de).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: error.path().to_string(),
            span: Some(create_span(
                &content,
                error.inner().line(),
                error.inner().column(),
            )),
            message: error.inner().to_string(),
        })?;

        Ok(result)
    }
}
