use crate::config::{Config, ExtendsFrom, PartialConfig};
use crate::errors::ConfigError;
use crate::format::Format;
use crate::layer::Layer;
use crate::source::Source;
use serde::Serialize;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tracing::trace;

#[derive(Serialize)]
pub struct ConfigLoadResult<T: Config> {
    /// Final configuration, after all layers are merged.
    pub config: T,

    /// Partial layers, in order of declaration and extension.
    pub layers: Vec<Layer<T>>,
}

#[derive(Default)]
pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    sources: Vec<Source>,
    root: Option<PathBuf>,
}

impl<T: Config> ConfigLoader<T> {
    /// Create a new config loader with the provided source format.
    pub fn new() -> Self {
        ConfigLoader {
            _config: PhantomData,
            sources: vec![],
            root: None,
        }
    }

    /// Add a code snippet source to load.
    pub fn code<S: TryInto<String>>(
        &mut self,
        code: S,
        format: Format,
    ) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::code(code, format)?);

        Ok(self)
    }

    /// Add a file source to load.
    pub fn file<S: TryInto<PathBuf>>(&mut self, path: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path, true)?);

        Ok(self)
    }

    /// Add a file source to load but don't error if the file doesn't exist.
    pub fn file_optional<S: TryInto<PathBuf>>(
        &mut self,
        path: S,
    ) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path, false)?);

        Ok(self)
    }

    /// Add a URL source to load.
    pub fn url<S: TryInto<String>>(&mut self, url: S) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::url(url)?);

        Ok(self)
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
        trace!("Loading {} configuration", T::META.name);

        let layers = self.parse_into_layers(&self.sources, context, false)?;
        let partial = self.merge_layers(&layers, context)?.finalize(context)?;

        // Validate the final result before moving on
        partial
            .validate(context)
            .map_err(|error| ConfigError::Validator {
                config: match layers.last() {
                    Some(last) => self.get_location(&last.source).to_owned(),
                    None => T::META.name.to_owned(),
                },
                error,
            })?;

        Ok(ConfigLoadResult {
            config: T::from_partial(partial),
            layers,
        })
    }

    /// Load, parse, and merge all sources into a partial configuration
    /// with the provided context. This does not inherit default values,
    /// environment variables, or run a final validation.
    ///
    /// Partials can be converted to full with [`Config::from_partial`].
    pub fn load_partial(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        trace!("Loading partial {} configuration", T::META.name);

        let layers = self.parse_into_layers(&self.sources, context, false)?;
        let partial = self.merge_layers(&layers, context)?;

        Ok(partial)
    }

    /// Set the project root directory, for use within error messages.
    pub fn set_root<P: AsRef<Path>>(&mut self, root: P) -> &mut Self {
        self.root = Some(root.as_ref().to_path_buf());
        self
    }

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

            trace!(source = ?source, "Extending additional source");

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

        self.parse_into_layers(&sources, context, true)
    }

    fn get_location<'l>(&self, source: &'l Source) -> &'l str {
        match source {
            Source::Code { .. } => T::META.name,
            Source::File { path, .. } => {
                let rel_path = if let Some(root) = &self.root {
                    if let Ok(other_path) = path.strip_prefix(root) {
                        other_path
                    } else {
                        path
                    }
                } else {
                    path
                };

                rel_path.to_str().unwrap_or(T::META.name)
            }
            Source::Url { url, .. } => url,
        }
    }

    fn merge_layers(
        &self,
        layers: &[Layer<T>],
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        trace!("Merging partial layers into a final result");

        // All `None` by default
        let mut merged = T::Partial::default();

        // Then apply other layers in order
        for layer in layers {
            merged.merge(context, layer.partial.clone())?;
        }

        Ok(merged)
    }

    fn parse_into_layers(
        &self,
        sources_to_parse: &[Source],
        context: &<T::Partial as PartialConfig>::Context,
        extending: bool,
    ) -> Result<Vec<Layer<T>>, ConfigError> {
        let mut layers: Vec<Layer<T>> = vec![];

        if !extending {
            trace!("Parsing sources into partial layers");
        }

        for source in sources_to_parse {
            trace!(source = ?source, "Parsing source");

            // Determine the source location for use in error messages
            let location = self.get_location(source);

            // Parse the source into a parial
            let partial: T::Partial = source.parse(location)?;

            // Validate before continuing so we ensure the values are correct
            partial
                .validate(context)
                .map_err(|error| ConfigError::Validator {
                    config: location.to_owned(),
                    error,
                })?;

            if let Some(extends_from) = partial.extends_from() {
                layers.extend(self.extend_additional_layers(context, source, &extends_from)?);
            }

            trace!(source = ?source, "Created layer from source");

            layers.push(Layer {
                partial,
                source: source.clone(),
            });
        }

        Ok(layers)
    }
}
