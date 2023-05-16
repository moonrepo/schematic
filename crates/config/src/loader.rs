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

    pub fn code<S: TryInto<String>>(&mut self, code: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::code(code)?);

        Ok(self)
    }

    pub fn file<S: TryInto<PathBuf>>(&mut self, path: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path)?);

        Ok(self)
    }

    pub fn source<S: AsRef<str>>(&mut self, value: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::new(value.as_ref(), None)?);

        Ok(self)
    }

    pub fn url<S: TryInto<String>>(&mut self, url: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::url(url)?);

        Ok(self)
    }

    pub fn load(&mut self) -> Result<ConfigLoadResult<T>, ConfigError> {
        let context = <T::Partial as PartialConfig>::Context::default();

        self.load_with_context(&context)
    }

    pub fn load_with_context(
        &mut self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<ConfigLoadResult<T>, ConfigError> {
        let sources_to_parse = mem::take(&mut self.sources);
        let (partial_layers, resolved_sources) = self.parse_into_layers(sources_to_parse)?;
        let partial = self.merge_layers(partial_layers, context)?;
        let config = T::from_partial(partial);

        config.validate().map_err(ConfigError::Validator)?;

        Ok(ConfigLoadResult {
            config,
            format: self.format,
            sources: resolved_sources,
        })
    }

    fn extend_additional_layers(
        &mut self,
        parent_source: &Source,
        extends_from: &ExtendsFrom,
    ) -> Result<(Vec<T::Partial>, Vec<Source>), ConfigError> {
        let mut sources = vec![];

        let mut extend_source = |value: &str| {
            let source = Source::new(value, Some(parent_source))?;

            // Extending from code is not possible
            if matches!(source, Source::Code { .. }) {
                return Err(ConfigError::ExtendsFromNoCode);
            }

            sources.push(source);

            Ok(())
        };

        match extends_from {
            ExtendsFrom::String(value) => {
                extend_source(value)?;
            }
            ExtendsFrom::List(values) => {
                for value in values.iter() {
                    extend_source(value)?;
                }
            }
        };

        self.parse_into_layers(sources)
    }

    fn merge_layers(
        &self,
        layers: Vec<T::Partial>,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        // All `None` by default
        let mut merged = T::Partial::default();

        // First layer should be the defaults
        merged.merge(T::Partial::default_values(context)?);

        // Then apply other layers in order
        for layer in layers {
            merged.merge(layer);
        }

        Ok(merged)
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
