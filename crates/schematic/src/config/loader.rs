use crate::helpers::strip_bom;
use crate::internal;

use super::cacher::{BoxedCacher, Cacher, MemoryCache};
use super::configs::{Config, PartialConfig};
use super::error::ConfigError;
#[cfg(feature = "extends")]
use super::extender::ExtendsFrom;
use super::layer::Layer;
use super::source::{Source, SourceFormat};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{instrument, trace};

/// The result of loading a configuration. Includes the final configuration,
/// and all layers that were loaded.
#[derive(Serialize)]
pub struct ConfigLoadResult<T: Config> {
    /// Final configuration, after all layers are merged.
    pub config: T,

    /// Partial layers, from the lowest precedence to the highest: the default
    /// values first, then each source in order of declaration and extension,
    /// and environment variables last. The default values and environment
    /// variables are only included when they set something, and are read
    /// against what the sources picked, like the variant of an enum.
    pub layers: Vec<Layer<T>>,
}

/// Tracks the sources being loaded, so that an `extends` cycle is an error rather
/// than infinite recursion, and a source extended more than once is only loaded once.
#[derive(Default)]
struct SourceTracker {
    /// Sources currently being loaded, from the outermost inward,
    /// as a key and the location to report in an error.
    stack: Vec<(String, String)>,

    /// Keys of every source loaded so far.
    loaded: HashSet<String>,
}

impl SourceTracker {
    fn key(source: &Source) -> Option<String> {
        match source {
            Source::Code { .. } | Source::Defaults | Source::EnvVars | Source::Memory { .. } => {
                None
            }
            Source::File { path, .. } => Some(
                fs::canonicalize(path)
                    .unwrap_or_else(|_| path.to_path_buf())
                    .to_string_lossy()
                    .into_owned(),
            ),
            #[cfg(feature = "url")]
            Source::Url { url } => Some(url.to_owned()),
        }
    }
}

/// A source to load, with its partial when the source is in memory.
type SourceEntry<P> = (Source, Option<P>);

/// A system for loading configuration from multiple sources in multiple formats,
/// and generating a final result after merging and validating layers.
pub struct ConfigLoader<T: Config> {
    _config: PhantomData<T>,
    cacher: Mutex<BoxedCacher>,
    formats: Vec<Arc<dyn SourceFormat<T::Partial>>>,
    help: Option<String>,
    name: String,
    sources: Vec<SourceEntry<T::Partial>>,
    root: Option<PathBuf>,
}

impl<T: Config> Default for ConfigLoader<T> {
    fn default() -> Self {
        ConfigLoader {
            _config: PhantomData,
            cacher: Mutex::new(Box::<MemoryCache>::default()),
            formats: vec![],
            help: None,
            name: T::schema_name().unwrap_or_else(|| "<unknown>".into()),
            sources: vec![],
            root: None,
        }
    }
}

impl<T: Config> ConfigLoader<T> {
    /// Create a new config loader and auto-register formats based on enables features.
    pub fn new() -> Self {
        // Only mutated when a format feature registers one below
        #[allow(unused_mut)]
        let mut loader = ConfigLoader::default();

        #[cfg(feature = "json")]
        loader.add_format(super::formats::JsonFormat::default());

        #[cfg(feature = "pkl")]
        loader.add_format(super::formats::PklFormat::default());

        #[cfg(feature = "ron")]
        loader.add_format(super::formats::RonFormat::default());

        #[cfg(feature = "toml")]
        loader.add_format(super::formats::TomlFormat::default());

        #[cfg(feature = "yaml")]
        loader.add_format(super::formats::YamlFormat::default());

        loader
    }

    /// Add a format to parse sources based on file name/extension detection.
    pub fn add_format(&mut self, format: impl SourceFormat<T::Partial> + 'static) -> &mut Self {
        self.formats.push(Arc::new(format));
        self
    }

    /// Add a code snippet source to load, with a required file name.
    pub fn code<S: TryInto<String>, P: TryInto<PathBuf>>(
        &mut self,
        code: S,
        path: P,
    ) -> Result<&mut Self, ConfigError> {
        self.source(Source::code(code, path)?)
    }

    /// Add a file source to load.
    pub fn file<P: TryInto<PathBuf>>(&mut self, path: P) -> Result<&mut Self, ConfigError> {
        self.source(Source::file(path, true)?)
    }

    /// Add a file source to load but don't error if the file doesn't exist.
    pub fn file_optional<S: TryInto<PathBuf>>(
        &mut self,
        path: S,
    ) -> Result<&mut Self, ConfigError> {
        self.source(Source::file(path, false)?)
    }

    /// Add a partial configuration held in memory, like one parsed from CLI
    /// arguments, as a layer named for use within error messages.
    ///
    /// It's loaded like any other source: merged in the order it was added,
    /// validated, and able to extend other sources. Like any other source, it
    /// sits below environment variables.
    pub fn partial<N: Into<String>>(&mut self, name: N, partial: T::Partial) -> &mut Self {
        self.sources
            .push((Source::Memory { name: name.into() }, Some(partial)));
        self
    }

    /// Add a custom source. A [`Source::Memory`] is an error, as only
    /// [`ConfigLoader::partial`] can provide its partial, as are
    /// [`Source::Defaults`] and [`Source::EnvVars`], which the loader adds.
    pub fn source(&mut self, source: Source) -> Result<&mut Self, ConfigError> {
        match source {
            Source::Memory { name } => {
                return Err(ConfigError::MemoryWithoutPartial(name));
            }
            Source::Defaults | Source::EnvVars => {
                return Err(ConfigError::LoaderOnlySource(source.as_str().into()));
            }
            _ => {}
        };

        self.sources.push((source, None));

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

        let sources =
            self.parse_into_layers(&self.sources, context, &mut SourceTracker::default(), false)?;

        // Validation is reported at the last source
        #[cfg(feature = "validate")]
        let last_source = sources.last().map(|layer| layer.source.clone());

        // Environment variables go over the sources, and default values under
        // them, each read against what the sources picked
        let applied = internal::apply_layers(self.merge_layers(&sources, context)?, context)?;
        let mut layers = Vec::with_capacity(sources.len() + 2);

        if let Some(partial) = applied.defaults {
            layers.push(Layer {
                partial,
                source: Source::Defaults,
            });
        }

        layers.extend(sources);

        if let Some(partial) = applied.env {
            layers.push(Layer {
                partial,
                source: Source::EnvVars,
            });
        }

        let partial = applied.partial.finalize(context)?;

        // Validate the final result before moving on
        #[cfg(feature = "validate")]
        {
            partial
                .validate(context, true)
                .map_err(|error| self.map_validator_error(error, last_source.as_ref()))?;
        }

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
    #[instrument(name = "load_partial_config", skip_all)]
    pub fn load_partial(
        &self,
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<T::Partial, ConfigError> {
        trace!(config = &self.name, "Loading partial configuration");

        let layers =
            self.parse_into_layers(&self.sources, context, &mut SourceTracker::default(), false)?;
        let partial = self.merge_layers(&layers, context)?.unwrap_or_default();

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
        tracker: &mut SourceTracker,
    ) -> Result<Vec<Layer<T>>, ConfigError> {
        let mut sources: Vec<SourceEntry<T::Partial>> = vec![];

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

            sources.push((source, None));

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

        self.parse_into_layers(&sources, context, tracker, true)
    }

    fn get_location<'l>(&'l self, source: &'l Source) -> &'l str {
        match source {
            Source::Code { .. } => &self.name,
            Source::Defaults => "default values",
            Source::EnvVars => "environment variables",
            Source::Memory { name } => name,
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

    /// Merge the layers in order, starting from the first, or `None` without
    /// any. Starting from the partial's [`Default`] instead would be the same
    /// for a struct, but an enum's holds a variant, which a layer of the same
    /// variant would be merged into, rather than standing alone.
    #[instrument(skip_all)]
    fn merge_layers(
        &self,
        layers: &[Layer<T>],
        context: &<T::Partial as PartialConfig>::Context,
    ) -> Result<Option<T::Partial>, ConfigError> {
        trace!(
            config = &self.name,
            "Merging partial layers into a final result"
        );

        let mut layers = layers.iter();
        let Some(first) = layers.next() else {
            return Ok(None);
        };
        let mut merged = first.partial.clone();

        for layer in layers {
            merged.merge(context, layer.partial.clone())?;
        }

        Ok(Some(merged))
    }

    #[instrument(skip_all)]
    fn parse_into_layers(
        &self,
        sources_to_parse: &[SourceEntry<T::Partial>],
        #[allow(unused_variables)] context: &<T::Partial as PartialConfig>::Context,
        tracker: &mut SourceTracker,
        extended: bool,
    ) -> Result<Vec<Layer<T>>, ConfigError> {
        let mut layers: Vec<Layer<T>> = vec![];

        for (source, memory) in sources_to_parse {
            let key = SourceTracker::key(source);

            // Sources added to the loader are always loaded, while extended
            // sources are checked for cycles, and are only loaded once
            if extended && let Some(key) = &key {
                if let Some(index) = tracker.stack.iter().position(|(k, _)| k == key) {
                    let mut chain = tracker.stack[index..]
                        .iter()
                        .map(|(_, location)| location.to_owned())
                        .collect::<Vec<_>>();

                    chain.push(self.get_location(source).to_owned());

                    return Err(ConfigError::ExtendsCycle(chain));
                }

                if tracker.loaded.contains(key) {
                    trace!(
                        config = &self.name,
                        source = source.as_str(),
                        "Skipping source that was already loaded"
                    );

                    continue;
                }
            }

            if let Some(key) = &key {
                tracker.loaded.insert(key.to_owned());
                tracker
                    .stack
                    .push((key.to_owned(), self.get_location(source).to_owned()));
            }

            trace!(
                config = &self.name,
                source = source.as_str(),
                "Creating layer from source"
            );

            // Parse the source into a partial
            let partial: T::Partial = self
                .parse_source(source, memory.as_ref())
                .map_err(|error| self.map_parser_error(error, source))?;

            // Validate before continuing so we ensure the values are correct
            #[cfg(feature = "validate")]
            {
                partial
                    .validate(context, false)
                    .map_err(|error| self.map_validator_error(error, Some(source)))?;
            }

            #[cfg(feature = "extends")]
            if let Some(extends_from) = partial.extends_from() {
                layers.extend(self.extend_additional_layers(
                    context,
                    source,
                    &extends_from,
                    tracker,
                )?);
            }

            if key.is_some() {
                tracker.stack.pop();
            }

            layers.push(Layer {
                partial,
                source: source.clone(),
            });
        }

        Ok(layers)
    }

    #[instrument(skip_all)]
    fn parse_source(
        &self,
        source: &Source,
        memory: Option<&T::Partial>,
    ) -> Result<T::Partial, ConfigError> {
        let (content, cache_path): (Cow<'_, str>, Option<PathBuf>) = match source {
            Source::Code { code, .. } => (Cow::Borrowed(strip_bom(code)), None),
            Source::Defaults | Source::EnvVars => {
                return Err(ConfigError::LoaderOnlySource(source.as_str().into()));
            }
            // Already a partial, so there's nothing to parse
            Source::Memory { name } => {
                return memory
                    .cloned()
                    .ok_or_else(|| ConfigError::MemoryWithoutPartial(name.to_owned()));
            }
            Source::File { path, required } => {
                let content = if path.exists() {
                    fs::read_to_string(path).map_err(|error| ConfigError::ReadFileFailed {
                        path: path.to_path_buf(),
                        error: Box::new(error),
                    })?
                } else {
                    if *required {
                        return Err(ConfigError::MissingFile(path.to_path_buf()));
                    }

                    return Ok(T::Partial::default());
                };

                (Cow::Owned(strip_bom(&content).to_owned()), None)
            }
            #[cfg(feature = "url")]
            Source::Url { url } => {
                use crate::helpers::is_secure_url;

                if !is_secure_url(url) {
                    return Err(ConfigError::HttpsOnly(url.to_owned()));
                }

                let mut cacher = self.cacher.lock().unwrap();

                let handle_http_error = |error: ureq::Error| ConfigError::ReadUrlFailed {
                    url: url.to_owned(),
                    error: Box::new(error),
                };

                let content = if let Some(cache) = cacher.read(url)? {
                    cache
                } else {
                    let body = ureq::get(url)
                        .call()
                        .map_err(handle_http_error)?
                        .body_mut()
                        .read_to_string()
                        .map_err(handle_http_error)?;

                    cacher.write(url, &body)?;

                    body
                };

                (
                    Cow::Owned(strip_bom(&content).to_owned()),
                    cacher.get_file_path(url)?,
                )
            }
        };

        for format in &self.formats {
            if format.should_parse(source) {
                return format.parse(source, &content, cache_path.as_deref());
            }
        }

        Err(ConfigError::NoMatchingFormat {
            src: source.as_str().to_owned(),
            ext: source.get_file_ext().unwrap_or("(none)").into(),
        })
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
