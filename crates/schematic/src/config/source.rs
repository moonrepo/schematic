use crate::config::cacher::BoxedCacher;
use crate::config::errors::ConfigError;
use crate::config::format::Format;
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::instrument;

/// Source from which to load a configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Source {
    /// Inline code snippet of the configuration.
    Code { code: String, format: Format },

    /// File system path to the configuration.
    File {
        path: PathBuf,
        format: Format,
        required: bool,
    },

    /// Secure URL to the configuration.
    Url { url: String, format: Format },
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

            if parent_source.is_none() {
                return Source::file(value, true);
            }

            if let Source::File {
                path: parent_path, ..
            } = parent_source.unwrap()
            {
                let mut path = PathBuf::from(value);

                // Not absolute, so prefix with parent
                if !path.has_root() {
                    path = parent_path.parent().unwrap().join(path);
                }

                return Source::file(path, true);
            } else {
                return Err(ConfigError::ExtendsFromParentFileOnly);
            }
        }

        Err(ConfigError::ExtendsFromNoCode)
    }

    /// Create a new code snippet source.
    pub fn code<T: TryInto<String>>(code: T, format: Format) -> Result<Source, ConfigError> {
        let code: String = code.try_into().map_err(|_| ConfigError::InvalidCode)?;

        Ok(Source::Code { code, format })
    }

    /// Create a new file source with the provided path.
    pub fn file<T: TryInto<PathBuf>>(path: T, required: bool) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;

        Ok(Source::File {
            format: Format::detect(path.to_str().unwrap_or_default())?,
            path,
            required,
        })
    }

    /// Create a new URL source with the provided URL.
    pub fn url<T: TryInto<String>>(url: T) -> Result<Source, ConfigError> {
        let url: String = url.try_into().map_err(|_| ConfigError::InvalidUrl)?;

        Ok(Source::Url {
            format: Format::detect(&url)?,
            url,
        })
    }

    /// Parse the source contents according to the required format.
    #[instrument(name = "parse_config_source", skip(cacher, help), fields(source = ?self))]
    pub fn parse<D>(
        &self,
        location: &str,
        cacher: &mut BoxedCacher,
        help: Option<&str>,
    ) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        let handle_error = |error: crate::config::ParserError| ConfigError::Parser {
            config: location.to_owned(),
            error,
            help: help.map(|h| h.to_owned()),
        };

        match self {
            Source::Code { code, format } => format
                .parse(code.to_owned(), location)
                .map_err(handle_error),
            Source::File {
                path,
                format,
                required,
            } => {
                let content = if path.exists() {
                    fs::read_to_string(path).map_err(|error| ConfigError::ReadFileFailed {
                        path: path.to_path_buf(),
                        error,
                    })?
                } else {
                    if *required {
                        return Err(ConfigError::MissingFile(path.to_path_buf()));
                    }

                    "".into()
                };

                format.parse(content, location).map_err(handle_error)
            }
            Source::Url { url, format } => {
                if !is_secure_url(url) {
                    return Err(ConfigError::HttpsOnly(url.to_owned()));
                }

                #[cfg(feature = "url")]
                {
                    let handle_reqwest_error = |error: reqwest::Error| ConfigError::ReadUrlFailed {
                        url: url.to_owned(),
                        error,
                    };

                    let content = if let Some(cache) = cacher.read(url)? {
                        cache
                    } else {
                        let body = reqwest::blocking::get(url)
                            .map_err(handle_reqwest_error)?
                            .text()
                            .map_err(handle_reqwest_error)?;

                        cacher.write(url, &body)?;

                        body
                    };

                    format.parse(content, location).map_err(handle_error)
                }

                #[cfg(not(feature = "url"))]
                {
                    panic!("Parsing a URL requires the `url` feature!");
                }
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Source::Code { .. } => "<code>",
            Source::File { path, .. } => path.to_str().unwrap_or_default(),
            Source::Url { url, .. } => url,
        }
    }
}

/// Returns true if the value ends in a supported file extension.
pub fn is_source_format(value: &str) -> bool {
    value.ends_with(".json")
        || value.ends_with(".toml")
        || value.ends_with(".yaml")
        || value.ends_with(".yml")
}

/// Returns true if the value looks like a file, by checking for `file://`,
/// path separators, or supported file extensions.
pub fn is_file_like(value: &str) -> bool {
    value.starts_with("file://")
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with('.')
        || value.contains('/')
        || value.contains('\\')
        || value.contains('.')
}

/// Returns true if the value looks like a URL, by checking for `http://`, `https://`, or `www`.
pub fn is_url_like(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("www")
}

/// Returns true if the value is a secure URL, by checking for `https://`. This check can be
/// bypassed for localhost URLs.
pub fn is_secure_url(value: &str) -> bool {
    if value.contains("127.0.0.1") || value.contains("//localhost") {
        return true;
    }

    value.starts_with("https://")
}
