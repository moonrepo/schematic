use super::validator::ValidatorError;
// use crate::config::parser::ParserError;
use crate::format::UnsupportedFormatError;
use miette::Diagnostic;
use starbase_styles::{Style, Stylize};
use std::fmt::Display;
use std::path::PathBuf;
use thiserror::Error;

/// All configuration based errors.
#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
    #[error(transparent)]
    Handler(#[from] Box<HandlerError>),

    #[error(transparent)]
    UnsupportedFormat(#[from] Box<UnsupportedFormatError>),

    #[diagnostic(code(config::extends::no_source_code))]
    #[error("Unable to extend, expected a file path or secure URL.")]
    ExtendsFromNoCode,

    #[diagnostic(code(config::extends::only_parent_file))]
    #[error("Extending from a file is only allowed if the parent source is also a file.")]
    ExtendsFromParentFileOnly,

    #[diagnostic(code(config::code::invalid))]
    #[error("Invalid code block used as a source.")]
    InvalidCode,

    #[diagnostic(code(config::default::invalid))]
    #[error("Invalid default value. {0}")]
    InvalidDefaultValue(String),

    #[diagnostic(code(config::file::invalid))]
    #[error("Invalid file path used as a source.")]
    InvalidFile,

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,
}

impl From<HandlerError> for ConfigError {
    fn from(e: HandlerError) -> ConfigError {
        ConfigError::Handler(Box::new(e))
    }
}

impl From<UnsupportedFormatError> for ConfigError {
    fn from(e: UnsupportedFormatError) -> ConfigError {
        ConfigError::UnsupportedFormat(Box::new(e))
    }
}

/// Error for handler functions.
#[derive(Error, Debug, Diagnostic)]
#[error("{0}")]
pub struct HandlerError(pub String);

impl HandlerError {
    pub fn new<T: Display>(message: T) -> Self {
        Self(message.to_string())
    }
}
