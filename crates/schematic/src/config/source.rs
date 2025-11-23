use super::error::ConfigError;
use crate::helpers::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;

/// Source from which to load a configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Source {
    /// Inline code snippet of the configuration.
    Code { path: PathBuf, code: String },

    /// File system path to the configuration.
    File { path: PathBuf, required: bool },

    /// Secure URL to the configuration.
    #[cfg(feature = "url")]
    Url { url: String },
}

impl Source {
    /// Create a new source with the provided value. Will attempt to infer the
    /// type of source based on characters within the value:
    ///
    /// - Will be a URL, if the value starts with `http://`, `https://`, or `www`.
    /// - Will be a file, if the file ends in an extension, or contains path separators.
    /// - Otherwise will be an error.
    pub fn new(value: &str, parent_source: Option<&Source>) -> Result<Source, ConfigError> {
        // Extending from a URL is allowed from any parent source
        #[cfg(feature = "url")]
        if is_url_like(value) {
            return Source::url(value);
        }

        // Extending from a file is only allowed from file parent sources
        if is_file_like(value) {
            let value = if let Some(stripped) = value.strip_prefix("file://") {
                stripped
            } else {
                value
            };

            return match parent_source {
                None => Source::file(value, true),
                Some(Source::File {
                    path: parent_path, ..
                }) => {
                    let mut path = PathBuf::from(value);

                    // Not absolute, so prefix with parent
                    if !path.has_root() {
                        path = parent_path.parent().unwrap().join(path);
                    }

                    Source::file(path, true)
                }
                Some(_) => Err(ConfigError::ExtendsFromParentFileOnly),
            };
        }

        Err(ConfigError::ExtendsFromNoCode)
    }

    /// Create a new code snippet source.
    pub fn code<T: TryInto<String>, P: TryInto<PathBuf>>(
        code: T,
        path: P,
    ) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;
        let code: String = code.try_into().map_err(|_| ConfigError::InvalidCode)?;

        Ok(Source::Code { path, code })
    }

    /// Create a new file source with the provided path.
    pub fn file<P: TryInto<PathBuf>>(path: P, required: bool) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;

        Ok(Source::File { path, required })
    }

    /// Create a new URL source with the provided URL.
    #[cfg(feature = "url")]
    pub fn url<T: TryInto<String>>(url: T) -> Result<Source, ConfigError> {
        let url: String = url.try_into().map_err(|_| ConfigError::InvalidUrl)?;

        Ok(Source::Url { url })
    }

    /// Return a file extension (without period) for the source if one is available.
    pub fn get_file_ext(&self) -> Option<&str> {
        match self {
            Self::Code { path, .. } | Self::File { path, .. } => {
                path.extension().and_then(|name| name.to_str())
            }
            #[cfg(feature = "url")]
            Self::Url { url, .. } => extract_file_ext(url),
        }
    }

    /// Return a file name for the source.
    pub fn get_file_name(&self) -> &str {
        match self {
            Self::Code { path, .. } | Self::File { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown"),
            #[cfg(feature = "url")]
            Self::Url { url, .. } => extract_file_name(url),
        }
    }

    /// Return the source as a string, either as a file path or URL.
    pub fn as_str(&self) -> &str {
        match self {
            Source::Code { path, .. } | Source::File { path, .. } => {
                path.to_str().unwrap_or_default()
            }
            #[cfg(feature = "url")]
            Source::Url { url, .. } => url,
        }
    }
}

/// Parses a source into a specific format.
pub trait SourceFormat<T: DeserializeOwned> {
    /// Should this instance parse the provided source?
    fn should_parse(&self, source: &Source) -> bool;

    /// Parse the source contents and return the deserialized value.
    fn parse(&self, source: &Source, content: &str) -> Result<T, ConfigError>;
}
