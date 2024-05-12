use crate::config::parser::ParserError;
use crate::config::validator::ValidatorError;
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

    #[diagnostic(code(config::enums::invalid_fallback))]
    #[error("Invalid fallback variant {}, unable to parse type.", .0.style(Style::Symbol))]
    EnumInvalidFallback(String),

    #[diagnostic(code(config::enums::unknown_variant))]
    #[error("Unknown enum variant {}.", .0.style(Style::Id))]
    EnumUnknownVariant(String),

    #[diagnostic(code(config::code::extends))]
    #[error("Unable to extend, expected a file path or URL.")]
    ExtendsFromNoCode,

    #[diagnostic(code(config::file::extends))]
    #[error("Extending from a file is only allowed if the parent source is also a file.")]
    ExtendsFromParentFileOnly,

    #[diagnostic(code(config::code::invalid))]
    #[error("Invalid raw code used as a source.")]
    InvalidCode,

    #[diagnostic(code(config::default::invalid))]
    #[error("Invalid default value. {0}")]
    InvalidDefault(String),

    #[diagnostic(code(config::file::invalid))]
    #[error("Invalid file path used as a source.")]
    InvalidFile,

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

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,

    #[cfg(feature = "url")]
    #[diagnostic(code(config::url::read_failed))]
    #[error("Failed to read URL {}.", .url.style(Style::Url))]
    ReadUrlFailed {
        url: String,
        #[source]
        error: Box<reqwest::Error>,
    },

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed, received {}.", .0.style(Style::Url))]
    HttpsOnly(String),

    #[diagnostic(code(config::format::unsupported))]
    #[error("Unsupported format for {0}, expected {1}.")]
    UnsupportedFormat(String, String),

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
            ConfigError::Validator { error: inner, .. } => {
                push_end();
                message.push_str(&inner.to_full_string());
            }
            _ => {}
        };

        message.trim().to_string()
    }
}

impl From<HandlerError> for ConfigError {
    fn from(e: HandlerError) -> ConfigError {
        ConfigError::Handler(Box::new(e))
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
