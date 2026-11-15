use super::error::ConfigError;
#[cfg(feature = "extends")]
use super::extender::ExtendsFrom;
#[cfg(feature = "validate")]
use super::validator::*;
use schematic_types::Schematic;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;

/// Represents a partial configuration of the base [`Config`], with all settings marked as optional
/// by wrapping the values in [`Option`].
pub trait PartialConfig:
    Clone + Default + DeserializeOwned + Schematic + Serialize + Sized
{
    type Context: Default;

    /// Return a partial configuration with values populated with default values for settings
    /// marked with `#[setting(default)]`. Unmarked settings will be [`None`].
    ///
    /// If a default value fails to parse or cast into the correct type, an error is returned.
    fn default_values(_context: &Self::Context) -> Result<Option<Self>, ConfigError> {
        Ok(None)
    }

    /// Return a partial configuration with values populated from environment variables
    /// for settings marked with `#[setting(env)]`. Unmarked settings will be [`None`].
    ///
    /// If an environment variable does not exist, the value will be [`None`]. If
    /// the variable fails to parse or cast into the correct type, an error is returned.
    #[cfg(feature = "env")]
    fn env_values() -> Result<Option<Self>, ConfigError> {
        Ok(None)
    }

    /// Internal use only, use [`env_values`] instead.
    ///
    /// Return environment variable values with a prefix, from a parent's
    /// `#[setting(nested, env_prefix)]`, applied to keys derived from setting
    /// names. A hand-written partial reads keys of its own, which a parent's
    /// prefix doesn't apply to, so this defaults to [`env_values`].
    #[cfg(feature = "env")]
    #[doc(hidden)]
    fn env_values_with_prefix(_prefix: Option<&str>) -> Result<Option<Self>, ConfigError> {
        Self::env_values()
    }

    /// When a setting is marked as extendable with `#[setting(extend)]`, this returns
    /// [`ExtendsFrom`] with the extended sources, either a list of strings or a single string.
    /// When no setting is extendable, this returns [`None`].
    #[cfg(feature = "extends")]
    fn extends_from(&self) -> Option<ExtendsFrom> {
        None
    }

    /// Finalize the partial configuration, once every layer has been merged into it, by running
    /// the `#[setting(transform)]` functions and finalizing nested configs.
    ///
    /// Default values and environment variables are not applied here, as they're layers of their
    /// own: [`ConfigLoader`](crate::ConfigLoader) merges [`PartialConfig::default_values`] below
    /// every source, and [`PartialConfig::env_values`] above them, before finalizing. A nested
    /// config's are part of its parent's, so it's finalized as is, except for the items of a
    /// collection and the values of a variant, which nothing above applied them to.
    fn finalize(self, _context: &Self::Context) -> Result<Self, ConfigError> {
        Ok(self)
    }

    /// Merge another partial configuration into this one and clone values when applicable. The
    /// following merge strategies are applied:
    ///
    /// - Current [`None`] values are replaced with the next value if [`Some`].
    /// - Current [`Some`] values are merged with the next value if [`Some`],
    ///   using the merge function from `#[setting(merge)]`.
    fn merge(&mut self, _context: &Self::Context, _next: Self) -> Result<(), ConfigError> {
        Ok(())
    }

    /// Recursively validate the configuration with the provided context.
    /// Validation should be done on the final state, after merging partials.
    #[cfg(feature = "validate")]
    fn validate(&self, context: &Self::Context, finalizing: bool) -> Result<(), ConfigError> {
        if let Err(errors) =
            self.validate_with_path(context, finalizing, super::path::Path::default())
        {
            return Err(ConfigError::Validator {
                location: String::new(),
                error: Box::new(ValidatorError { errors }),
                help: None,
            });
        }

        Ok(())
    }

    /// Internal use only, use [`validate`] instead.
    #[cfg(feature = "validate")]
    #[doc(hidden)]
    fn validate_with_path(
        &self,
        _context: &Self::Context,
        _finalizing: bool,
        _path: super::path::Path,
    ) -> Result<(), Vec<ValidateError>> {
        Ok(())
    }
}

/// Represents the final configuration, with all settings populated with a value.
pub trait Config: Sized + Schematic {
    type Partial: PartialConfig;

    /// Return default values for the partial configuration.
    ///
    /// # Panics
    ///
    /// Panics if a default value could not be generated, as this is
    /// infallible from the perspective of [`Default`].
    fn default_partial() -> Self::Partial {
        let context = <<Self as Config>::Partial as PartialConfig>::Context::default();

        <<Self as Config>::Partial as PartialConfig>::default_values(&context)
            .expect("Failed to generate default values.")
            .unwrap_or_default()
    }

    /// Convert a partial configuration into a full configuration, with all values populated.
    fn from_partial(partial: Self::Partial) -> Self;

    /// Return a map of all settings and their metadata for the configuration.
    fn settings() -> ConfigSettingMap {
        BTreeMap::default()
    }
}

/// Represents an enumerable setting for use within a [`Config`].
pub trait ConfigEnum: Sized + Schematic {
    /// Return a list of all variants for the enum. Only unit variants are supported.
    fn variants() -> Vec<Self>;
}

/// Represents metadata about a setting within a configuration.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetting {
    pub env_key: Option<String>,
    pub nested: Option<ConfigSettingMap>,
    pub type_alias: String,
}

impl ConfigSetting {
    pub fn new(type_alias: impl AsRef<str>) -> Self {
        Self {
            env_key: None,
            nested: None,
            type_alias: type_alias.as_ref().to_string(),
        }
    }

    pub fn env(mut self, env_key: impl AsRef<str>) -> Self {
        self.env_key = Some(env_key.as_ref().to_string());
        self
    }

    pub fn nested(mut self, nested: ConfigSettingMap) -> Self {
        self.nested = Some(nested);
        self
    }
}

pub type ConfigSettingMap = BTreeMap<String, ConfigSetting>;
