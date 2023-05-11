use crate::error::ConfigError;
use serde::{de::DeserializeOwned, Serialize};
use starbase_utils::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "yaml")]
    Yaml,
}

impl SourceFormat {
    pub fn parse<D>(&self, content: String) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        let data: D = match self {
            #[cfg(feature = "json")]
            SourceFormat::Json => {
                serde_json::from_str(&content).map_err(ConfigError::JsonParseFailed)?
            }

            #[cfg(feature = "yaml")]
            SourceFormat::Yaml => {
                let mut value: serde_yaml::Value =
                    serde_yaml::from_str(&content).map_err(ConfigError::YamlParseFailed)?;

                // Applies anchors/aliases/references
                value.apply_merge().map_err(ConfigError::YamlParseFailed)?;

                D::deserialize(value).map_err(ConfigError::YamlParseFailed)?
            }
        };

        Ok(data)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Source {
    Code { code: String },
    File { path: PathBuf },
    Url { url: String },
}

impl Source {
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

    pub async fn parse<D>(&self, format: SourceFormat) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        let content = match self {
            Source::Code { code } => code.to_owned(),
            Source::File { path } => {
                if path.exists() {
                    fs::read_file(path)?
                } else {
                    return Err(ConfigError::MissingFile(path.to_path_buf()));
                }
            }
            Source::Url { url } => reqwest::get(url).await?.text().await?,
        };

        format.parse(content)
    }
}
