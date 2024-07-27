use super::parser::ParserError;
use super::validator::ValidatorError;
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

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed, received {}.", .0.style(Style::Url))]
    HttpsOnly(String),

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

    #[diagnostic(code(config::file::missing), help("Is the path absolute?"))]
    #[error("File path {} does not exist.", .0.style(Style::Path))]
    MissingFile(PathBuf),

    #[diagnostic(code(config::file::read_failed))]
    #[error("Failed to read file {}.", .path.style(Style::Path))]
    ReadFileFailed {
        path: PathBuf,
        #[source]
        error: Box<std::io::Error>,
    },

    #[cfg(feature = "url")]
    #[diagnostic(code(config::url::read_failed))]
    #[error("Failed to read URL {}.", .url.style(Style::Url))]
    ReadUrlFailed {
        url: String,
        #[source]
        error: Box<reqwest::Error>,
    },

    // Parser
    #[diagnostic(code(config::parse::failed))]
    #[error("Failed to parse {}.", .config.style(Style::File))]
    Parser {
        config: String,

        // Required to display the code snippet!
        // Because of this, we can't wrap in `Box`.
        #[diagnostic_source]
        #[source]
        error: ParserError,

        #[help]
        help: Option<String>,
    },

    // Validator
    #[diagnostic(code(config::validate::failed))]
    #[error("Failed to validate {}.", .config.style(Style::File))]
    Validator {
        config: String,

        // This includes a vertical red line which we don't want!
        // #[diagnostic_source]
        #[source]
        error: Box<ValidatorError>,

        #[help]
        help: Option<String>,
    },
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
