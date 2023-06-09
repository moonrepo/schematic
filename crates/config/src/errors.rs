use crate::validator::ValidatorError;
use miette::{Diagnostic, NamedSource, SourceSpan};
use starbase_styles::{Style, Stylize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),

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

    #[diagnostic(code(config::env::invalid))]
    #[error("Invalid environment variable {}. {1}", .0.style(Style::Symbol))]
    InvalidEnvVar(String, String),

    #[diagnostic(code(config::file::invalid))]
    #[error("Invalid file path used as a source.")]
    InvalidFile,

    #[diagnostic(code(config::file::missing), help("Is the path absolute?"))]
    #[error("File path {} does not exist.", .0.style(Style::Path))]
    MissingFile(PathBuf),

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed, received {}.", .0.style(Style::Url))]
    HttpsOnly(String),

    #[diagnostic(code(config::format::unsupported))]
    #[error("Unsupported format for {0}, expected {1}.")]
    UnsupportedFormat(String, String),

    // IO
    #[diagnostic(code(config::fs))]
    #[error("Failed to read source file.")]
    Io(#[from] std::io::Error),

    // HTTP
    #[diagnostic(code(config::http))]
    #[error("Failed to download source from URL.")]
    Http(#[from] reqwest::Error),

    // Parser
    #[diagnostic(code(config::parse::failed))]
    #[error("Failed to parse {}", .config.style(Style::File))]
    Parser {
        config: String,

        #[diagnostic_source]
        #[source]
        error: ParserError,
    },

    // Validator
    #[diagnostic(code(config::validate::failed))]
    #[error("Failed to validate {}", .config.style(Style::File))]
    Validator {
        config: String,

        // This includes the vertical red line which we don't want!
        // #[diagnostic_source]
        #[source]
        error: ValidatorError,
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
            ConfigError::Io(inner) => {
                push_end();
                message.push_str(&inner.to_string());
            }
            ConfigError::Http(inner) => {
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

/// Error related to serde parsing.
#[derive(Error, Debug, Diagnostic)]
#[error("{}{} {message}", .path.style(Style::Id), ":".style(Style::MutedLight))]
#[diagnostic(severity(Error))]
pub struct ParserError {
    #[source_code]
    pub content: NamedSource,

    pub message: String,

    pub path: String,

    #[label("Fix this")]
    pub span: Option<SourceSpan>,
}
