use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use miette::NamedSource;
use serde::de::DeserializeOwned;

pub fn parse<D>(name: &str, content: &str) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    let result: D = ron::from_str(content).map_err(|error| {
        // Extract position from error
        let position = error.position;
        let line = position.line;
        let column = position.col;

        ParserError {
            content: NamedSource::new(name, content.to_owned()),
            path: String::new(), // RON doesn't provide field path info
            span: Some(super::create_span(content, line, column)),
            message: error.code.to_string(),
        }
    })?;

    Ok(result)
}
