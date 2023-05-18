use crate::error::{ConfigError, ParserError};
use miette::{NamedSource, SourceOffset, SourceSpan};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

fn create_span(content: &str, line: usize, column: usize) -> SourceSpan {
    let offset = SourceOffset::from_location(content, line, column).offset();
    let length = 0;

    (offset, length).into()
}

fn create_parser_error<T: Display>(
    content: &str,
    loc: Option<(usize, usize)>,
    error: &serde_path_to_error::Error<T>,
) -> ParserError {
    ParserError {
        content: NamedSource::new("todo", content.to_owned()),
        path: error.path().to_string(),
        span: loc.map(|(line, column)| create_span(content, line, column)),
        error: error.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(feature = "yaml")]
    Yaml,
}

impl SourceFormat {
    pub fn parse<D>(&self, content: String) -> Result<D, ParserError>
    where
        D: DeserializeOwned,
    {
        let data: D = match self {
            #[cfg(feature = "json")]
            SourceFormat::Json => {
                let de = &mut serde_json::Deserializer::from_str(&content);

                serde_path_to_error::deserialize(de).map_err(|error| {
                    create_parser_error(
                        &content,
                        Some((error.inner().line(), error.inner().column())),
                        &error,
                    )
                })?
            }

            #[cfg(feature = "toml")]
            SourceFormat::Toml => {
                let de = toml::Deserializer::new(&content);

                serde_path_to_error::deserialize(de).map_err(|error| ParserError {
                    content: NamedSource::new("todo", content.to_owned()),
                    path: error.path().to_string(),
                    span: error.inner().span().map(|s| s.into()),
                    error: error.to_string(),
                })?
            }

            #[cfg(feature = "yaml")]
            SourceFormat::Yaml => {
                use serde::de::IntoDeserializer;

                // First pass, convert string to value
                let de = serde_yaml::Deserializer::from_str(&content);
                let mut result: serde_yaml::Value =
                    serde_path_to_error::deserialize(de).map_err(|error| {
                        create_parser_error(
                            &content,
                            error
                                .inner()
                                .location()
                                .map(|s| (s.line(), s.column()).into()),
                            &error,
                        )
                    })?;

                // Applies anchors/aliases/references
                result.apply_merge().map_err(|error| ParserError {
                    content: NamedSource::new("todo", content.to_owned()),
                    path: String::new(),
                    span: error.location().map(|s| (s.line(), s.column()).into()),
                    error: error.to_string(),
                })?;

                // Second pass, convert value to struct
                let de = result.into_deserializer();

                serde_path_to_error::deserialize(de).map_err(|error| {
                    create_parser_error(
                        &content,
                        error
                            .inner()
                            .location()
                            .map(|s| (s.line(), s.column()).into()),
                        &error,
                    )
                })?
            }
        };

        Ok(data)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Code { code: String },
    Defaults,
    Env,
    File { path: PathBuf },
    Url { url: String },
}

impl Source {
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
                return Source::file(value);
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

                return Source::file(path);
            } else {
                return Err(ConfigError::ExtendsFromParentFileOnly);
            }
        }

        Source::code(value)
    }

    pub fn code<T: TryInto<String>>(code: T) -> Result<Source, ConfigError> {
        let code: String = code.try_into().map_err(|_| ConfigError::InvalidCode)?;

        Ok(Source::Code { code })
    }

    pub fn file<T: TryInto<PathBuf>>(path: T) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;

        Ok(Source::File { path })
    }

    pub fn url<T: TryInto<String>>(url: T) -> Result<Source, ConfigError> {
        let url: String = url.try_into().map_err(|_| ConfigError::InvalidUrl)?;

        if !url.starts_with("https://") {
            return Err(ConfigError::HttpsOnly);
        }

        Ok(Source::Url { url })
    }

    pub fn parse<D>(&self, format: SourceFormat, label: &str) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        format
            .parse(match self {
                Source::Code { code } => code.to_owned(),
                Source::File { path } => {
                    if !path.exists() {
                        return Err(ConfigError::MissingFile(path.to_path_buf()));
                    }

                    fs::read_to_string(path)?
                }
                Source::Url { url } => reqwest::blocking::get(url)?.text()?,
                _ => unreachable!(),
            })
            .map_err(|error| ConfigError::Parser {
                config: label.to_owned(),
                content: error.content,
                error: error.error,
                path: error.path,
                span: error.span,
            })
    }
}

pub fn is_file_like(value: &str) -> bool {
    value.starts_with("file://")
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with('.')
        || value.contains('/')
        || value.contains('\\')
        || value.ends_with(".json")
        || value.ends_with(".toml")
        || value.ends_with(".yaml")
        || value.ends_with(".yml")
}

pub fn is_url_like(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://") || value.starts_with("www")
}
