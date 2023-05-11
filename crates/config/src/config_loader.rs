use crate::config::{Config, PartialConfig};
use crate::error::ConfigError;
use crate::source::{Source, SourceFormat};
use std::marker::PhantomData;
use std::path::PathBuf;

pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    source_format: SourceFormat,
    sources: Vec<Source>,
}

impl<T: Config> ConfigLoader<T> {
    pub fn new(source_format: SourceFormat) -> Self {
        ConfigLoader {
            _config: PhantomData,
            source_format,
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

    pub async fn load(&self) -> Result<(), ConfigError> {
        let partial_layers = self.parse_into_layers().await?;

        Ok(())
    }

    async fn parse_into_layers(&self) -> Result<Vec<T::Partial>, ConfigError> {
        let mut layers: Vec<T::Partial> = vec![];

        // First layer should be the defaults
        layers.push(T::Partial::default_values());

        // Sources would then overrides the defaults in sequence
        for source in &self.sources {
            let partial: T::Partial = source.parse(self.source_format).await?;
            layers.push(partial);
        }

        Ok(layers)
    }
}

// fn test() {
//     let config: SomeConfig = ConfigLoader::new(SourceFormat::Yaml)
//         .file("foo.yml")?
//         .url("https://example.com/bar.yml")?
//         .load()?;
// }
