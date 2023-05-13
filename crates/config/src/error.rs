use miette::Diagnostic;
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

    #[diagnostic(code(config::env::invalid))]
    #[error("Invalid environment variable {0}. {1}")]
    InvalidEnvVar(String, String),

    #[diagnostic(code(config::file::invalid))]
    #[error("Invalid file path used as a source.")]
    InvalidFile,

    #[diagnostic(code(config::file::missing), help("Ensure the path is absolute?"))]
    #[error("File path {0} does not exist.")]
    MissingFile(PathBuf),

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed.")]
    HttpsOnly,

    // JSON
    #[cfg(feature = "json")]
    #[diagnostic(code(config::json::parse_failed))]
    #[error("Failed to parse JSON source.")]
    JsonParseFailed(#[source] serde_json::Error),

    // TOML
    #[cfg(feature = "toml")]
    #[diagnostic(code(config::toml::parse_failed))]
    #[error("Failed to parse TOML source.")]
    TomlParseFailed(#[source] toml::de::Error),

    // YAML
    #[cfg(feature = "yaml")]
    #[diagnostic(code(config::yaml::parse_failed))]
    #[error("Failed to parse YAML source.")]
    YamlParseFailed(#[source] serde_yaml::Error),

    // IO
    #[diagnostic(code(config::fs))]
    #[error("Failed to read file.")]
    Io(#[from] std::io::Error),

    // HTTP
    #[diagnostic(code(config::http))]
    #[error("Failed to download source from URL.")]
    Http(#[from] reqwest::Error),
}
