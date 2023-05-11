use miette::Diagnostic;
use starbase_utils::fs::FsError;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
    #[diagnostic(code(config::code::invalid))]
    #[error("Invalid raw code used as a source.")]
    InvalidCode,

    #[diagnostic(code(config::file::invalid))]
    #[error("Invalid file path used as a source.")]
    InvalidFile,

    #[diagnostic(code(config::url::invalid))]
    #[error("Invalid URL used as a source.")]
    InvalidUrl,

    #[diagnostic(code(config::url::https_only))]
    #[error("Only secure URLs are allowed.")]
    HttpsOnly,

    // YAML
    #[cfg(feature = "yaml")]
    #[diagnostic(code(config::yaml::parse_failed))]
    #[error("Failed to parse YAML source.")]
    YamlParseFailed(#[source] serde_yaml::Error),

    // Inherited
    #[diagnostic(code(config::fs))]
    #[error(transparent)]
    Fs(#[from] FsError),

    #[diagnostic(code(config::http))]
    #[error("Failed to download source from URL.")]
    Http(#[from] reqwest::Error),
}
