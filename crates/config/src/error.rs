use crate::validator::ValidatorError;
use miette::Diagnostic;
use starbase_styles::{Style, Stylize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),

    #[diagnostic(code(config::code::extends))]
    #[error("Unable to extend from a code based source.")]
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

    #[diagnostic(code(config::file::missing), help("Ensure the path is absolute?"))]
    #[error("File path {} does not exist.", .0.style(Style::Path))]
    MissingFile(PathBuf),

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed.")]
    HttpsOnly,

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
    #[error("Failed to parse config.")]
    Parser(
        #[diagnostic_source]
        #[source]
        ParserError,
    ),

    // Validator
    #[diagnostic(code(config::validate::failed))]
    #[error("Failed to validate {config}.")]
    Validator {
        config: String,

        #[diagnostic_source]
        #[source]
        error: ValidatorError,
    },
}

impl ConfigError {
    pub fn to_full_string(&self) -> String {
        let mut message = self.to_string();

        match self {
            ConfigError::Io(inner) => {
                message.push(' ');
                message.push_str(&inner.to_string());
            }
            ConfigError::Http(inner) => {
                message.push(' ');
                message.push_str(&inner.to_string());
            }
            ConfigError::Parser(inner) => {
                message.push(' ');
                message.push_str(&inner.to_full_string());
            }
            ConfigError::Validator { error: inner, .. } => {
                message.push(' ');
                message.push_str(&inner.to_full_string());
            }
            _ => {}
        };

        message
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParserError {
    #[cfg(feature = "json")]
    #[diagnostic(code(parse::json::failed))]
    #[error("Invalid setting {}:", .path.style(Style::Id))]
    Json {
        #[source]
        error: serde_json::Error,
        path: String,
    },

    #[cfg(feature = "toml")]
    #[diagnostic(code(parse::toml::failed))]
    #[error("Invalid setting {}:", .path.style(Style::Id))]
    Toml {
        #[source]
        error: toml::de::Error,
        path: String,
    },

    #[cfg(feature = "yaml")]
    #[diagnostic(code(parse::yaml::failed))]
    #[error("Invalid setting {}:", .path.style(Style::Id))]
    Yaml {
        #[source]
        error: serde_yaml::Error,
        path: String,
    },

    #[cfg(feature = "yaml")]
    #[diagnostic(code(parse::yaml::extended))]
    #[error("Failed to apply YAML anchors and references.")]
    YamlExtended {
        #[source]
        error: serde_yaml::Error,
    },
}

impl ParserError {
    pub fn to_full_string(&self) -> String {
        let mut message = self.to_string();
        message.push_str("\n  ");

        match self {
            #[cfg(feature = "json")]
            ParserError::Json { error, .. } => {
                message.push_str(&error.to_string());
            }
            #[cfg(feature = "toml")]
            ParserError::Toml { error, .. } => {
                message.push_str(error.message());
            }
            #[cfg(feature = "yaml")]
            ParserError::Yaml { error, .. } | ParserError::YamlExtended { error } => {
                message.push_str(&error.to_string());
            }
        };

        message
    }
}
