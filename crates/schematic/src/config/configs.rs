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
        Self::env_values_with_prefix(None)
    }

    /// Internal use only, use [`env_values`] instead.
    #[cfg(feature = "env")]
    #[doc(hidden)]
    fn env_values_with_prefix(_prefix: Option<&str>) -> Result<Option<Self>, ConfigError> {
        Ok(None)
    }

    /// When a setting is marked as extendable with `#[setting(extend)]`, this returns
    /// [`ExtendsFrom`] with the extended sources, either a list of strings or a single string.
    /// When no setting is extendable, this returns [`None`].
    #[cfg(feature = "extends")]
    fn extends_from(&self) -> Option<ExtendsFrom> {
        None
    }

    /// Finalize the partial configuration by consuming it and populating all fields with a value.
    /// Defaults values from [`PartialConfig::default_values`] will be applied first, followed
    /// by merging the current partial, and lastly environment variable values from
    /// [`PartialConfig::env_values`].
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
    fn validate(&self, context: &Self::Context, finalize: bool) -> Result<(), ConfigError> {
        if let Err(errors) =
            self.validate_with_path(context, finalize, super::path::Path::default())
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
        _finalize: bool,
        _path: super::path::Path,
    ) -> Result<(), Vec<ValidateError>> {
        Ok(())
    }
}

/// Represents the final configuration, with all settings populated with a value.
pub trait Config: Sized + Schematic {
    type Partial: PartialConfig;

    /// Return default values for the partial configuration.
    fn default_partial() -> Self::Partial {
        let context = <<Self as Config>::Partial as PartialConfig>::Context::default();

        <<Self as Config>::Partial as PartialConfig>::default_values(&context)
            .unwrap()
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

pub type ConfigSettingMap = BTreeMap<String, ConfigSetting>;
