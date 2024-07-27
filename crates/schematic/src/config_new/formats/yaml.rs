use super::super::parser::ParserError;
use super::create_span;
use serde::de::{DeserializeOwned, IntoDeserializer};

pub fn parse<D>(content: String) -> Result<D, ParserError>
where
    D: DeserializeOwned,
{
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
    })
}
