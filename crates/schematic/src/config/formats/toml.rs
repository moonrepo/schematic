use crate::config::error::{ConfigError, HandlerError};
use crate::config::parser::ParserError;
use miette::NamedSource;
use serde::de::DeserializeOwned;

pub fn parse<D>(name: &str, content: &str) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    let de = toml::Deserializer::parse(content)
        .map_err(|error| ConfigError::Handler(Box::new(HandlerError::new(error.to_string()))))?;

    let result: D = serde_path_to_error::deserialize(de).map_err(|error| ParserError {
        content: NamedSource::new(name, content.to_owned()),
        path: error.path().to_string(),
        span: error.inner().span().map(|s| s.into()),
        message: error.inner().message().to_owned(),
    })?;

    Ok(result)
}
