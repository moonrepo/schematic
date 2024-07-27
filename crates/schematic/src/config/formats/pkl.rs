use super::super::parser::ParserError;
use super::create_span;
use miette::NamedSource;
use serde::de::DeserializeOwned;

pub fn parse<D>(name: &str, content: &str) -> Result<D, ParserError>
where
    D: DeserializeOwned,
{
    use rpkl::pkl::PklSerialize;

    // Based on `rpkl::from_config`
    let mut evaluator = rpkl::api::Evaluator::new().expect("TODO");
    let ast = evaluator
        .evaluate_module(source_path.expect("TODO").to_path_buf())
        .expect("TODO")
        .serialize_pkl_ast()
        .expect("TODO");
    let mut de = rpkl::pkl::Deserializer::from_pkl_map(&ast);

    serde_path_to_error::deserialize(&mut de).map_err(|error| ParserError {
        // content: NamedSource::new(location, content.to_owned()),
        content: content.to_owned(),
        path: error.path().to_string(),
        span: None,
        message: error.inner().to_string(),
    })
}
