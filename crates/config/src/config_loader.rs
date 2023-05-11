use crate::config::{Config, PartialConfig};
use crate::error::ConfigError;
use crate::source::{Source, SourceFormat};
use serde::Serialize;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ConfigLoadResult<T: Config> {
    pub config: T,
    pub format: SourceFormat,
    pub sources: Vec<Source>,
}

pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    format: SourceFormat,
    sources: Vec<Source>,
}

impl<T: Config> ConfigLoader<T> {
    pub fn new(format: SourceFormat) -> Self {
        ConfigLoader {
            _config: PhantomData,
            format,
            sources: vec![],
        }
    }

    pub fn code<P: TryInto<String>>(&mut self, code: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::code(code)?);

        Ok(self)
    }

    pub fn file<P: TryInto<PathBuf>>(&mut self, path: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path)?);

        Ok(self)
    }

    pub fn url<P: TryInto<String>>(&mut self, url: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::url(url)?);

        Ok(self)
    }

    pub async fn load(mut self) -> Result<ConfigLoadResult<T>, ConfigError> {
        let (partial_layers, resolved_sources) = self.parse_into_layers().await?;
        let partial = self.merge_layers(partial_layers);
        let config = T::from_partial(partial)?;

        Ok(ConfigLoadResult {
            config,
            format: self.format,
            sources: resolved_sources,
        })
    }

    fn merge_layers(&self, layers: Vec<T::Partial>) -> T::Partial {
        // All `None` by default
        let mut merged = T::Partial::default();

        for layer in layers {
            merged.merge(layer);
        }

        merged
    }

    async fn parse_into_layers(&mut self) -> Result<(Vec<T::Partial>, Vec<Source>), ConfigError> {
        let mut layers: Vec<T::Partial> = vec![];
        let mut sources: Vec<Source> = vec![];

        // First layer should be the defaults
        layers.push(T::Partial::default_values());

        // Sources would then override the defaults in sequence
        for source in self.sources.drain(0..) {
            let partial: T::Partial = source.parse(self.format).await?;

            layers.push(partial);
            sources.push(source);
        }

        Ok((layers, sources))
    }
}
