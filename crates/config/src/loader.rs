use crate::config::{Config, ExtendsFrom, PartialConfig};
use crate::error::ConfigError;
use crate::layer::Layer;
use crate::source::{Source, SourceFormat};
use serde::Serialize;
use starbase_styles::color;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ConfigLoadResult<T: Config> {
    /// Final configuration, after all layers are merged.
    pub config: T,

    /// Format of sources.
    pub format: SourceFormat,

    /// Partial layers, in order of declaration and extension.
    pub layers: Vec<Layer<T>>,
}

pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    format: SourceFormat,
    label: String,
    sources: Vec<Source>,
}

impl<T: Config> ConfigLoader<T> {
    /// Create a new config loader with the provided source format.
    pub fn new(format: SourceFormat) -> Self {
        let meta = T::META;

        ConfigLoader {
            _config: PhantomData,
            format,
            label: if let Some(file) = &meta.file {
                color::file(file)
            } else {
                color::label(meta.name)
            },
            sources: vec![],
        }
    }

    #[cfg(feature = "json")]
    /// Create a new JSON config loader.
    pub fn json() -> Self {
        ConfigLoader::new(SourceFormat::Json)
    }

    #[cfg(feature = "toml")]
    /// Create a new TOML config loader.
    pub fn toml() -> Self {
        ConfigLoader::new(SourceFormat::Toml)
    }

    #[cfg(feature = "yaml")]
    /// Create a new YAML config loader.
    pub fn yaml() -> Self {
        ConfigLoader::new(SourceFormat::Yaml)
    }

    /// Add a code snippet source to load.
    pub fn code<S: TryInto<String>>(&mut self, code: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::code(code)?);

        Ok(self)
    }

    /// Add a file source to load.
    pub fn file<S: TryInto<PathBuf>>(&mut self, path: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path)?);

        Ok(self)
    }

    /// Add a source to load. Will attempt to infer the source type.
    pub fn source<S: AsRef<str>>(&mut self, value: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::new(value.as_ref(), None)?);

        Ok(self)
    }

    /// Add a URL source to load.
    pub fn url<S: TryInto<String>>(&mut self, url: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::url(url)?);

        Ok(self)
    }

    /// Set the label to include in error messages. By default will be the configuration
    /// struct name, or the `#[config(file = "...")]` attribute if set.
    pub fn label(&mut self, label: String) -> &mut Self {
        self.label = label;
        self
    }

    /// Load, parse, merge, and validate all sources into a final configuration.
    pub fn load(&self) -> Result<ConfigLoadResult<T>, ConfigError> {
        let context = <T::Partial as PartialConfig>::Context::default();

        self.load_with_context(&context)
    }

    /// Load, parse, merge, and validate all sources into a final configuration
    /// with the provided context. Context will be passed to all applicable
    /// default, merge, and validate functions defined with `#[setting]`.
    pub fn load_with_context(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<ConfigLoadResult<T>, ConfigError> {
        let layers = self.parse_into_layers(&self.sources)?;
        let partial = self.merge_layers(&layers, context)?;
        let config = T::from_partial(context, partial, true)?;

        config
            .validate(context)
            .map_err(|error| ConfigError::Validator {
                config: self.label.clone(),
                error,
            })?;

        Ok(ConfigLoadResult {
            config,
            format: self.format,
            layers,
        })
    }

    /// Load, parse, and merge all sources into a partial configuration
    /// with the provided context. Validation will _not_ be performed.
    ///
    /// Partials can be converted to full with [`Config::from_partial`].
    pub fn load_partial(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        let layers = self.parse_into_layers(&self.sources)?;
        let partial = self.merge_layers(&layers, context)?;

        Ok(partial)
    }

    fn extend_additional_layers(
        &self,
        parent_source: &Source,
        extends_from: &ExtendsFrom,
    ) -> Result<Vec<Layer<T>>, ConfigError> {
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

        self.parse_into_layers(&sources)
    }

    fn merge_layers(
        &self,
        layers: &[Layer<T>],
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        // All `None` by default
        let mut merged = T::Partial::default();

        // Then apply other layers in order
        for layer in layers {
            merged.merge(context, layer.partial.clone())?;
        }

        Ok(merged)
    }

    fn parse_into_layers(&self, sources_to_parse: &[Source]) -> Result<Vec<Layer<T>>, ConfigError> {
        let mut layers: Vec<Layer<T>> = vec![];

        for source in sources_to_parse {
            let partial: T::Partial = source.parse(self.format, &self.label)?;

            if let Some(extends_from) = partial.extends_from() {
                layers.extend(self.extend_additional_layers(source, &extends_from)?);
            }

            layers.push(Layer {
                partial,
                source: source.clone(),
            });
        }

        Ok(layers)
    }
}
