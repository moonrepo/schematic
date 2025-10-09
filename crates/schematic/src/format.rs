use crate::helpers::extract_ext;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported source configuration formats.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    // This is to simply handle the use case when no features are
    // enabled. If this doesn't exist, Rust errors with no variants.
    #[doc(hidden)]
    #[default]
    None,

    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "pkl")]
    Pkl,

    #[cfg(feature = "ron")]
    Ron,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(any(feature = "yaml", feature = "yml"))]
    Yaml,
}

impl Format {
    /// Detects a format from a provided value, either a file path or URL, by
    /// checking for a supported file extension.
    pub fn detect(value: &str) -> Result<Format, UnsupportedFormatError> {
        let mut available: Vec<&str> = vec![];
        let ext = extract_ext(value).unwrap_or_default();

        #[cfg(feature = "json")]
        {
            available.push("JSON");

            if ext == ".json" {
                return Ok(Format::Json);
            }
        }

        #[cfg(feature = "pkl")]
        {
            available.push("Pkl");

            if ext == ".pkl" {
                return Ok(Format::Pkl);
            }
        }

        #[cfg(feature = "ron")]
        {
            available.push("RON");

            if ext == ".ron" {
                return Ok(Format::Ron);
            }
        }

        #[cfg(feature = "toml")]
        {
            available.push("TOML");

            if ext == ".toml" {
                return Ok(Format::Toml);
            }
        }

        #[cfg(any(feature = "yaml", feature = "yml"))]
        {
            available.push("YAML");

            if ext == ".yaml" || ext == ".yml" {
                return Ok(Format::Yaml);
            }
        }

        Err(UnsupportedFormatError(
            value.to_owned(),
            available.join(", "),
        ))
    }

    pub fn is_json(&self) -> bool {
        #[cfg(feature = "json")]
        {
            matches!(self, Format::Json)
        }
        #[cfg(not(feature = "json"))]
        {
            false
        }
    }

    pub fn is_pkl(&self) -> bool {
        #[cfg(feature = "pkl")]
        {
            matches!(self, Format::Pkl)
        }
        #[cfg(not(feature = "pkl"))]
        {
            false
        }
    }

    pub fn is_ron(&self) -> bool {
        #[cfg(feature = "ron")]
        {
            matches!(self, Format::Ron)
        }
        #[cfg(not(feature = "ron"))]
        {
            false
        }
    }

    pub fn is_toml(&self) -> bool {
        #[cfg(feature = "toml")]
        {
            matches!(self, Format::Toml)
        }
        #[cfg(not(feature = "toml"))]
        {
            false
        }
    }

    pub fn is_yaml(&self) -> bool {
        #[cfg(any(feature = "yaml", feature = "yml"))]
        {
            matches!(self, Format::Yaml)
        }
        #[cfg(not(any(feature = "yaml", feature = "yml")))]
        {
            false
        }
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(code(config::format::unsupported))]
#[error("Unsupported format for {0}, expected {1}.")]
pub struct UnsupportedFormatError(String, String);
