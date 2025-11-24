use crate::config::error::ConfigError;
use crate::config::parser::ParserError;
use crate::config::source::*;
use miette::NamedSource;
use rpkl::{EvaluatorOptions, pkl::PklSerialize};
use serde::de::DeserializeOwned;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[allow(unused_imports)]
pub use rpkl::{api::external_reader::*, api::reader::*};

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

pub type PklFormatOptions = EvaluatorOptions;

#[derive(Default)]
pub struct PklFormat {
    options: PklFormatOptions,
}

impl PklFormat {
    pub fn new(options: PklFormatOptions) -> Self {
        Self { options }
    }
}

impl<T: DeserializeOwned> SourceFormat<T> for PklFormat {
    fn should_parse(&self, source: &Source) -> bool {
        source.get_file_ext() == Some("pkl")
    }

    fn parse(
        &self,
        source: &Source,
        content: &str,
        cache_path: Option<&Path>,
    ) -> Result<T, ConfigError> {
        check_pkl_installed()?;

        let Some(file_path) = cache_path.or_else(|| match source {
            Source::File { path, .. } => Some(path),
            _ => None,
        }) else {
            return Err(ConfigError::PklFileRequired);
        };

        let handle_error = |error: rpkl::Error| ConfigError::PklEvalFailed {
            path: file_path.to_path_buf(),
            error: Box::new(error),
        };

        // Based on `rpkl::from_config`
        let ast = rpkl::api::Evaluator::new_from_options(self.options.clone())
            .map_err(handle_error)?
            .evaluate_module(file_path)
            .map_err(handle_error)?
            .serialize_pkl_ast()
            .map_err(handle_error)?;

        let mut de = rpkl::pkl::Deserializer::from_pkl_map(&ast);

        let result: T = serde_path_to_error::deserialize(&mut de).map_err(|error| ParserError {
            content: NamedSource::new(source.get_file_name(), content.to_owned()),
            path: error.path().to_string(),
            span: None,
            message: error.inner().to_string(),
        })?;

        Ok(result)
    }
}
