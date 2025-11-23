use super::create_span;
use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use serde::de::{DeserializeOwned, IntoDeserializer};

#[allow(unreachable_code)]
pub fn parse<D>(name: &str, content: &str) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    #[cfg(feature = "yaml")]
    {
        fn create_parser_error(
            name: &str,
            content: &str,
            path: String,
            error: serde_yaml::Error,
        ) -> ParserError {
            ParserError {
                content: NamedSource::new(name, content.to_owned()),
                path,
                span: error
                    .location()
                    .map(|s| create_span(content, s.line(), s.column())),
                message: error.to_string(),
            }
        }

        // First pass, convert string to value
        let de = serde_yaml::Deserializer::from_str(content);

        let mut result: serde_yaml::Value =
            serde_path_to_error::deserialize(de).map_err(|error| {
                create_parser_error(name, content, error.path().to_string(), error.into_inner())
            })?;

        // Applies anchors/aliases/references
        result
            .apply_merge()
            .map_err(|error| create_parser_error(name, content, String::new(), error))?;

        // Second pass, convert value to struct
        let de = result.into_deserializer();

        let result: D = serde_path_to_error::deserialize(de).map_err(|error| {
            create_parser_error(name, content, error.path().to_string(), error.into_inner())
        })?;

        return Ok(result);
    }

    #[cfg(feature = "yml")]
    {
        fn create_parser_error(
            name: &str,
            content: &str,
            path: String,
            error: serde_yml::Error,
        ) -> ParserError {
            ParserError {
                content: NamedSource::new(name, content.to_owned()),
                path,
                span: error
                    .location()
                    .map(|s| create_span(content, s.line(), s.column())),
                message: error.to_string(),
            }
        }

        // First pass, convert string to value
        let de = serde_yml::Deserializer::from_str(content);

        let mut result: serde_yml::Value =
            serde_path_to_error::deserialize(de).map_err(|error| {
                create_parser_error(name, content, error.path().to_string(), error.into_inner())
            })?;

        // Applies anchors/aliases/references
        result
            .apply_merge()
            .map_err(|error| create_parser_error(name, content, String::new(), error))?;

        // Second pass, convert value to struct
        let de = result.into_deserializer();

        let result: D = serde_path_to_error::deserialize(de).map_err(|error| {
            create_parser_error(name, content, error.path().to_string(), error.into_inner())
        })?;

        return Ok(result);
    }

    panic!("Please enable the `yaml` or `yml` feature.");
}

#[derive(Default)]
pub struct YamlFormat;

impl<T: DeserializeOwned> SourceFormat<T> for YamlFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source
            .get_file_ext()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
    }

    fn parse(&self, source: &Source, content: &str) -> Result<T, ConfigError> {
        parse(source.get_file_name(), content)
    }
}
