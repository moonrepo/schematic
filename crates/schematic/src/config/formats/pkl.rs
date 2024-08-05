use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use miette::NamedSource;
use rpkl::pkl::PklSerialize;
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn parse<D>(name: &str, content: &str, file_path: Option<&Path>) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    let Some(file_path) = file_path else {
        return Err(ConfigError::PklFileRequired);
    };

    let handle_error = |error: rpkl::Error| ConfigError::PklEvalFailed {
        path: file_path.to_path_buf(),
        error: Box::new(error),
    };

    // Based on `rpkl::from_config`
    let ast = rpkl::api::Evaluator::new()
        .map_err(handle_error)?
        .evaluate_module(file_path.to_path_buf())
        .map_err(handle_error)?
        .serialize_pkl_ast()
        .map_err(handle_error)?;

    let mut de = rpkl::pkl::Deserializer::from_pkl_map(&ast);

    let result: D = serde_path_to_error::deserialize(&mut de).map_err(|error| ParserError {
        content: NamedSource::new(name, content.to_owned()),
        path: error.path().to_string(),
        span: None, // TODO
        message: error.inner().to_string(),
    })?;

    Ok(result)
}
