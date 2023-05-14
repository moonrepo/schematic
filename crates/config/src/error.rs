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
    #[error("{0}")]
    Parse(
        #[diagnostic_source]
        #[source]
        ParseError,
    ),

    // Validator
    #[diagnostic(code(config::validate::failed))]
    #[error("Failed to validate")]
    Validate {
        #[related]
        errors: Vec<ValidateType>,
    },
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseError {
    #[cfg(feature = "json")]
    #[diagnostic(code(parse::json::failed))]
    #[error("Failed to parse JSON setting `{path}`")]
    Json {
        #[source]
        error: serde_json::Error,
        path: String,
    },

    #[cfg(feature = "toml")]
    #[diagnostic(code(parse::toml::failed))]
    #[error("Failed to parse TOML setting `{path}`")]
    Toml {
        #[source]
        error: toml::de::Error,
        path: String,
    },

    #[cfg(feature = "yaml")]
    #[diagnostic(code(parse::yaml::failed))]
    #[error("Failed to parse YAML setting `{path}`")]
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

#[derive(Error, Debug, Diagnostic)]
#[error("{message}")]
pub struct ValidateError {
    message: String,
}

impl ValidateError {
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum ValidateType {
    #[error("Invalid setting `{path}`")]
    Rule {
        path: String,
        #[diagnostic_source]
        #[source]
        error: ValidateError,
    },
    #[error("Invalid setting `{path}`")]
    Nested {
        path: String,
        #[related]
        errors: Vec<ValidateError>,
    },
}

impl ValidateType {
    pub fn rule(path: &str, error: ValidateError) -> Self {
        ValidateType::Rule {
            path: path.to_owned(),
            error,
        }
    }

    pub fn nested(path: &str, errors: Vec<ValidateError>) -> Self {
        ValidateType::Nested {
            path: path.to_owned(),
            errors,
        }
    }
}
