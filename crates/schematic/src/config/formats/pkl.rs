use super::super::parser::ParserError;
use miette::NamedSource;
use rpkl::pkl::PklSerialize;
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn parse<D>(name: &str, content: &str, file_path: Option<&Path>) -> Result<D, ParserError>
where
    D: DeserializeOwned,
{
    // Based on `rpkl::from_config`
    let mut evaluator = rpkl::api::Evaluator::new().expect("TODO");
    let ast = evaluator
        .evaluate_module(file_path.expect("TODO").to_path_buf())
        .expect("TODO")
        .serialize_pkl_ast()
        .expect("TODO");
    let mut de = rpkl::pkl::Deserializer::from_pkl_map(&ast);

    serde_path_to_error::deserialize(&mut de).map_err(|error| ParserError {
        content: NamedSource::new(name, content.to_owned()),
        path: error.path().to_string(),
        span: None,
        message: error.inner().to_string(),
    })
}
