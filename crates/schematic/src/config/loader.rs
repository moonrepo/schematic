use super::cacher::{BoxedCacher, Cacher, MemoryCache};
use super::configs::{Config, PartialConfig};
use super::error::ConfigError;
#[cfg(feature = "extends")]
use super::extender::ExtendsFrom;
use super::layer::Layer;
use super::source::Source;
use crate::format::Format;
use serde::Serialize;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{instrument, trace};

/// The result of loading a configuration. Includes the final configuration,
/// and all layers that were loaded.
#[derive(Serialize)]
pub struct ConfigLoadResult<T: Config> {
    /// Final configuration, after all layers are merged.
    pub config: T,

    /// Partial layers, in order of declaration and extension.
    pub layers: Vec<Layer<T>>,
}

/// A system for loading configuration from multiple sources in multiple formats,
/// and generating a final result after merging and validating layers.
pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    cacher: Mutex<BoxedCacher>,
    help: Option<String>,
    name: String,
    sources: Vec<Source>,
    root: Option<PathBuf>,
}

impl<T: Config> ConfigLoader<T> {
    /// Create a new config loader.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ConfigLoader {
            _config: PhantomData,
            cacher: Mutex::new(Box::<MemoryCache>::default()),
            help: None,
            name: T::schema_name().unwrap_or_else(|| "<unknown>".into()),
            sources: vec![],
            root: None,
        }
    }

    /// Add explicit source code to load.
    pub fn code<S: TryInto<String>>(
        &mut self,
        code: S,
        format: Format,
    ) -> Result<&mut Self, ConfigError> {
        self.source(Source::code(code, format)?)
    }

    /// Add a file source to load.
    pub fn file<S: TryInto<PathBuf>>(&mut self, path: S) -> Result<&mut Self, ConfigError> {
        self.source(Source::file(path, true)?)
    }

    /// Add a file source to load but don't error if the file doesn't exist.
    pub fn file_optional<S: TryInto<PathBuf>>(
        &mut self,
        path: S,
    ) -> Result<&mut Self, ConfigError> {
        self.source(Source::file(path, false)?)
    }

    /// Add a custom source.
    pub fn source(&mut self, source: Source) -> Result<&mut Self, ConfigError> {
        self.sources.push(source);

        Ok(self)
    }

    /// Add a URL source to load.
    #[cfg(feature = "url")]
    pub fn url<S: TryInto<String>>(&mut self, url: S) -> Result<&mut Self, ConfigError> {
        self.source(Source::url(url)?)
    }

    /// Load, parse, merge, and validate all sources into a final configuration.
    pub fn load(&self) -> Result<ConfigLoadResult<T>, ConfigError> {
        let context = <T::Partial as PartialConfig>::Context::default();

        self.load_with_context(&context)
    }

    /// Load, parse, merge, and validate all sources into a final configuration
    /// with the provided context. Context will be passed to all applicable
    /// default, merge, and validate functions defined with `#[setting]`.
    #[instrument(name = "load_config", skip_all)]
    pub fn load_with_context(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<ConfigLoadResult<T>, ConfigError> {
        trace!(config = &self.name, "Loading configuration");

        let layers = self.parse_into_layers(&self.sources, context)?;
        let partial = self.merge_layers(&layers, context)?.finalize(context)?;

        // Validate the final result before moving on
        #[cfg(feature = "validate")]
        {
            partial.validate(context, true).map_err(|error| {
                self.map_validator_error(error, layers.last().map(|layer| &layer.source))
            })?;
        }

        Ok(ConfigLoadResult {
            config: T::from_partial(partial)?,
            layers,
        })
    }

    /// Load, parse, and merge all sources into a partial configuration
    /// with the provided context. This does not inherit default values,
    /// environment variables, or run a final validation.
    ///
    /// Partials can be converted to full with [`Config::from_partial`].
    #[instrument(name = "load_partial_config", skip_all)]
    pub fn load_partial(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        trace!(config = &self.name, "Loading partial configuration");

        let layers = self.parse_into_layers(&self.sources, context)?;
        let partial = self.merge_layers(&layers, context)?;

        Ok(partial)
    }

    /// Set a cacher instance that'll read and write the cache for URL requests.
    pub fn set_cacher(&mut self, cacher: impl Cacher + 'static) -> &mut Self {
        self.cacher = Mutex::new(Box::new(cacher));
        self
    }

    /// Set a string of help text to include in validation errors.
    pub fn set_help<H: AsRef<str>>(&mut self, help: H) -> &mut Self {
        self.help = Some(help.as_ref().to_owned());
        self
    }

    /// Set the project root directory, for use within error messages.
    pub fn set_root<P: AsRef<Path>>(&mut self, root: P) -> &mut Self {
        self.root = Some(root.as_ref().to_path_buf());
        self
    }

    #[cfg(feature = "extends")]
    #[instrument(skip_all)]
    fn extend_additional_layers(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
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

            trace!(
                config = &self.name,
                source = source.as_str(),
                "Extending additional source"
            );

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

        self.parse_into_layers(&sources, context)
    }

    fn get_location<'l>(&'l self, source: &'l Source) -> &'l str {
        match source {
            Source::Code { .. } => &self.name,
            Source::File { path, .. } => {
                let rel_path = if let Some(root) = &self.root {
                    path.strip_prefix(root).unwrap_or(path)
                } else {
                    path
                };

                rel_path.to_str().unwrap_or(&self.name)
            }
            #[cfg(feature = "url")]
            Source::Url { url, .. } => url,
        }
    }

    #[instrument(skip_all)]
    fn merge_layers(
        &self,
        layers: &[Layer<T>],
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        trace!(
            config = &self.name,
            "Merging partial layers into a final result"
        );

        // All `None` by default
        let mut merged = T::Partial::default();

        // Then apply other layers in order
        for layer in layers {
            merged.merge(context, layer.partial.clone())?;
        }

        Ok(merged)
    }

    #[instrument(skip_all)]
    fn parse_into_layers(
        &self,
        sources_to_parse: &[Source],
        #[cfg_attr(not(feature = "schema"), allow(unused_variables))]
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<Vec<Layer<T>>, ConfigError> {
        let mut layers: Vec<Layer<T>> = vec![];

        for source in sources_to_parse {
            trace!(
                config = &self.name,
                source = source.as_str(),
                "Creating layer from source"
            );

            // Parse the source into a partial
            let partial: T::Partial = {
                let mut cacher = self.cacher.lock().unwrap();

                source
                    .parse(&self.name, &mut cacher)
                    .map_err(|error| self.map_parser_error(error, source))?
            };

            // Validate before continuing so we ensure the values are correct
            #[cfg(feature = "validate")]
            {
                partial
                    .validate(context, false)
                    .map_err(|error| self.map_validator_error(error, Some(source)))?;
            }

            #[cfg(feature = "extends")]
            if let Some(extends_from) = partial.extends_from() {
                layers.extend(self.extend_additional_layers(context, source, &extends_from)?);
            }

            layers.push(Layer {
                partial,
                source: source.clone(),
            });
        }

        Ok(layers)
    }

    fn map_parser_error(&self, outer: ConfigError, source: &Source) -> ConfigError {
        match outer {
            ConfigError::Parser { error, .. } => ConfigError::Parser {
                location: self.get_location(source).to_owned(),
                error,
                help: self.help.clone(),
            },
            _ => outer,
        }
    }

    #[cfg(feature = "validate")]
    fn map_validator_error(&self, outer: ConfigError, source: Option<&Source>) -> ConfigError {
        match outer {
            ConfigError::Validator { error, .. } => ConfigError::Validator {
                location: source
                    .map(|src| self.get_location(src))
                    .unwrap_or(&self.name)
                    .to_owned(),
                error,
                help: self.help.clone(),
            },
            _ => outer,
        }
    }
}
