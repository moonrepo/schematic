use serde::de::DeserializeOwned;
use starbase_utils::fs;

use crate::error::ConfigError;
use std::path::PathBuf;

#[derive(Clone, Copy)]
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

pub enum Source {
    Code(String),
    File(PathBuf),
    Url(String),
}

impl Source {
    pub fn code<T: TryInto<String>>(code: T) -> Result<Source, ConfigError> {
        let code: String = code.try_into().map_err(|_| ConfigError::InvalidCode)?;

        Ok(Source::Code(code))
    }

    pub fn file<T: TryInto<PathBuf>>(path: T) -> Result<Source, ConfigError> {
        let path: PathBuf = path.try_into().map_err(|_| ConfigError::InvalidFile)?;

        Ok(Source::File(path))
    }

    pub fn url<T: TryInto<String>>(url: T) -> Result<Source, ConfigError> {
        let url: String = url.try_into().map_err(|_| ConfigError::InvalidUrl)?;

        if !url.starts_with("https://") {
            return Err(ConfigError::HttpsOnly);
        }

        Ok(Source::Url(url))
    }

    pub async fn parse<D>(&self, format: SourceFormat) -> Result<D, ConfigError>
    where
        D: DeserializeOwned,
    {
        format.parse(match self {
            Source::Code(code) => code.to_owned(),
            Source::File(path) => fs::read_file(path)?,
            Source::Url(url) => reqwest::get(url).await?.text().await?,
        })
    }
}
