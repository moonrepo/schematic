use crate::config::{Config, ExtendsFrom, PartialConfig};
use crate::error::ConfigError;
use crate::source::{Source, SourceFormat};
use serde::Serialize;
use std::marker::PhantomData;
use std::mem;
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

    #[cfg(feature = "json")]
    pub fn json() -> Self {
        ConfigLoader::new(SourceFormat::Json)
    }

    #[cfg(feature = "toml")]
    pub fn toml() -> Self {
        ConfigLoader::new(SourceFormat::Toml)
    }

    #[cfg(feature = "yaml")]
    pub fn yaml() -> Self {
        ConfigLoader::new(SourceFormat::Yaml)
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

    pub fn load(&mut self) -> Result<ConfigLoadResult<T>, ConfigError> {
        let sources_to_parse = mem::take(&mut self.sources);
        let (partial_layers, resolved_sources) = self.parse_into_layers(sources_to_parse)?;
        let partial = self.merge_layers(partial_layers);
        let config = T::from_partial(partial);

        Ok(ConfigLoadResult {
            config,
            format: self.format,
            sources: resolved_sources,
        })
    }

    fn extend_additional_layers(
        &mut self,
        parent_source: &Source,
        extends_from: &ExtendsFrom<'_>,
    ) -> Result<(Vec<T::Partial>, Vec<Source>), ConfigError> {
        let mut sources = vec![];

        match extends_from {
            ExtendsFrom::String(value) => {
                sources.push(Source::new(value, Some(parent_source))?);
            }
            ExtendsFrom::List(values) => {
                for value in values.iter() {
                    sources.push(Source::new(value, Some(parent_source))?);
                }
            }
        };

        self.parse_into_layers(sources)
    }

    fn merge_layers(&self, layers: Vec<T::Partial>) -> T::Partial {
        // All `None` by default
        let mut merged = T::Partial::default();

        // First layer should be the defaults
        merged.merge(T::Partial::default_values());

        // Then apply other layers in order
        for layer in layers {
            merged.merge(layer);
        }

        merged
    }

    fn parse_into_layers(
        &mut self,
        sources_to_parse: Vec<Source>,
    ) -> Result<(Vec<T::Partial>, Vec<Source>), ConfigError> {
        let mut layers: Vec<T::Partial> = vec![];
        let mut sources: Vec<Source> = vec![];

        for source in sources_to_parse {
            let partial: T::Partial = source.parse(self.format)?;

            if let Some(extends_from) = partial.extends_from() {
                let (extended_layers, extended_sources) =
                    self.extend_additional_layers(&source, &extends_from)?;

                layers.extend(extended_layers);
                sources.extend(extended_sources);
            }

            layers.push(partial);
            sources.push(source);
        }

        Ok((layers, sources))
    }
}
