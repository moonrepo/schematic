use super::merger::MergeError;
use super::parser::ParserError;
#[cfg(feature = "validate")]
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
    Merge(#[from] Box<MergeError>),

    #[error(transparent)]
    UnsupportedFormat(#[from] Box<UnsupportedFormatError>),

    #[diagnostic(code(config::enums::invalid_fallback))]
    #[error("Invalid fallback variant {}, unable to parse type.", .0.style(Style::Symbol))]
    EnumInvalidFallback(String),

    #[diagnostic(code(config::enums::unknown_variant))]
    #[error("Unknown enum variant {}.", .0.style(Style::Id))]
    EnumUnknownVariant(String),

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

    #[cfg(feature = "pkl")]
    #[diagnostic(code(config::pkl::failed))]
    #[error("Failed to evaluate Pkl file {}.", .path.style(Style::Path))]
    PklEvalFailed {
        path: PathBuf,
        #[source]
        error: Box<rpkl::Error>,
    },

    #[cfg(feature = "pkl")]
    #[diagnostic(code(config::pkl::file_required))]
    #[error("Pkl requires local file paths to evaluate, received a code snippet or URL.")]
    PklFileRequired,

    #[cfg(feature = "pkl")]
    #[diagnostic(code(config::pkl::binary_required))]
    #[error(
        "Pkl configuration requires the {} binary to be installed and available.\nLearn more: {}",
        "pkl".style(Style::Shell),
        "https://pkl-lang.org/main/current/pkl-cli/index.html".style(Style::Url)
    )]
    PklRequired,

    // Parser
    #[diagnostic(code(config::parse::failed))]
    #[error("Failed to parse {}.", .location.style(Style::File))]
    Parser {
        location: String,

        #[diagnostic_source]
        #[source]
        error: Box<ParserError>,

        #[help]
        help: Option<String>,
    },

    // Validator
    #[cfg(feature = "validate")]
    #[diagnostic(code(config::validate::failed))]
    #[error("Failed to validate {}.", .location.style(Style::File))]
    Validator {
        location: String,

        #[diagnostic_source]
        #[source]
        error: Box<ValidatorError>,

        #[help]
        help: Option<String>,
    },
}

impl ConfigError {
    /// Return a full error string, disregarding `miette` diagnostic structure.
    /// This is extremely useful for debugging and tests, and less for application use.
    pub fn to_full_string(&self) -> String {
        let mut message = self.to_string();

        let mut push_end = || {
            if !message.ends_with('\n') {
                if !message.ends_with('.') && !message.ends_with(':') {
                    message.push('.');
                }
                message.push(' ');
            }
        };

        match self {
            #[cfg(feature = "pkl")]
            ConfigError::PklEvalFailed { error: inner, .. } => {
                push_end();
                message.push_str(&inner.to_string());
            }
            ConfigError::ReadFileFailed { error: inner, .. } => {
                push_end();
                message.push_str(&inner.to_string());
            }
            #[cfg(feature = "url")]
            ConfigError::ReadUrlFailed { error: inner, .. } => {
                push_end();
                message.push_str(&inner.to_string());
            }
            ConfigError::Parser { error: inner, .. } => {
                push_end();
                message.push_str(&inner.to_string());
            }
            #[cfg(feature = "validate")]
            ConfigError::Validator { error: inner, .. } => {
                push_end();
                for error in &inner.errors {
                    message.push_str(format!("\n  {error}").as_str());
                }
            }
            _ => {}
        };

        message.trim().to_string()
    }
}

impl From<HandlerError> for ConfigError {
    fn from(error: HandlerError) -> ConfigError {
        ConfigError::Handler(Box::new(error))
    }
}

impl From<MergeError> for ConfigError {
    fn from(error: MergeError) -> ConfigError {
        ConfigError::Merge(Box::new(error))
    }
}

impl From<ParserError> for ConfigError {
    fn from(error: ParserError) -> ConfigError {
        ConfigError::Parser {
            location: String::new(),
            error: Box::new(error),
            help: None,
        }
    }
}

impl From<UnsupportedFormatError> for ConfigError {
    fn from(error: UnsupportedFormatError) -> ConfigError {
        ConfigError::UnsupportedFormat(Box::new(error))
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
