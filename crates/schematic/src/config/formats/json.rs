use super::super::parser::ParserError;
use super::create_span;
use miette::NamedSource;
use serde::de::DeserializeOwned;

pub fn parse<D>(name: &str, content: &str) -> Result<D, ParserError>
where
    D: DeserializeOwned,
{
    let de =
        &mut serde_json::Deserializer::from_str(if content.is_empty() { "{}" } else { &content });

    serde_path_to_error::deserialize(de).map_err(|error| ParserError {
        content: NamedSource::new(name, content.to_owned()),
        path: error.path().to_string(),
        span: Some(create_span(
            &content,
            error.inner().line(),
            error.inner().column(),
        )),
        message: error.inner().to_string(),
    })
}
