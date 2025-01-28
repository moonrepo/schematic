use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use miette::NamedSource;
use rpkl::pkl::PklSerialize;
use serde::de::DeserializeOwned;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static PKL_CHECKED: AtomicBool = AtomicBool::new(false);

fn check_pkl_installed() -> Result<(), ConfigError> {
    if !PKL_CHECKED.load(Ordering::Relaxed) {
        let paths = env::var_os("PATH").unwrap_or_default();
        let mut installed = false;

        for path in env::split_paths(&paths) {
            let file_path = path.join(if cfg!(windows) { "pkl.exe" } else { "pkl" });

            if file_path.exists() && file_path.is_file() {
                installed = true;
                break;
            }
        }

        if !installed {
            return Err(ConfigError::PklRequired);
        }

        PKL_CHECKED.store(true, Ordering::Release);
    }

    Ok(())
}

pub fn parse<D>(name: &str, content: &str, file_path: Option<&Path>) -> Result<D, ConfigError>
where
    D: DeserializeOwned,
{
    check_pkl_installed()?;

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
