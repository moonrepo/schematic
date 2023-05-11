use crate::error::ConfigError;
use serde::{de::DeserializeOwned, Serialize};
use starbase_utils::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    #[cfg(feature = "yaml")]
    Yaml,
}

impl SourceFormat {
    pub fn parse<D>(&self, content: String) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        let data: D = match self {
            #[cfg(feature = "yaml")]
            SourceFormat::Yaml => {
                serde_yaml::from_str(&content).map_err(ConfigError::YamlParseFailed)?
            }
        };

        Ok(data)
    }
}

#[derive(Serialize)]
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
