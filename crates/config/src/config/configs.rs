use crate::config::errors::ConfigError;
use crate::config::path::Path;
use crate::config::validator::ValidatorError;
use crate::derive_enum;
use schematic_types::Schematic;
use serde::{de::DeserializeOwned, Serialize};

/// Provides metadata about a configuration struct or enum.
pub struct Meta {
    /// Name of the struct.
    pub name: &'static str,
}

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
    fn default_values(context: &Self::Context) -> Result<Option<Self>, ConfigError>;

    /// Return a partial configuration with values populated from environment variables
    /// for settings marked with `#[setting(env)]`. Unmarked settings will be [`None`].
    ///
    /// If an environment variable does not exist, the value will be [`None`]. If
    /// the variable fails to parse or cast into the correct type, an error is returned.
    fn env_values() -> Result<Option<Self>, ConfigError>;

    /// When a setting is marked as extendable with `#[setting(extend)]`, this returns
    /// [`ExtendsFrom`] with the extended sources, either a list of strings or a single string.
    /// When no setting is extendable, this returns [`None`].
    fn extends_from(&self) -> Option<ExtendsFrom>;

    /// Finalize the partial configuration by consuming it and populating all fields with a value.
    /// Defaults values from [`PartialConfig::default_values`] will be applied first, followed
    /// by merging the current partial, and lastly environment variable values from
    /// [`PartialConfig::env_values`].
    fn finalize(self, context: &Self::Context) -> Result<Self, ConfigError>;

    /// Merge another partial configuration into this one and clone values when applicable. The
    /// following merge strategies are applied:
    ///
    /// - Current [`None`] values are replaced with the next value if [`Some`].
    /// - Current [`Some`] values are merged with the next value if [`Some`],
    ///     using the merge function from `#[setting(merge)]`.
    fn merge(&mut self, context: &Self::Context, next: Self) -> Result<(), ConfigError>;

    /// Recursively validate the configuration with the provided context.
    /// Validation should be done on the final state, after merging partials.
    fn validate(&self, context: &Self::Context) -> Result<(), ValidatorError> {
        self.validate_with_path(context, Path::default())
    }

    #[doc(hidden)]
    /// Internal use only, use [`validate`] instead.
    fn validate_with_path(
        &self,
        _context: &Self::Context,
        _path: Path,
    ) -> Result<(), ValidatorError> {
        Ok(())
    }
}

/// Represents the final configuration, with all settings populated with a value.
pub trait Config: Sized + Schematic {
    type Partial: PartialConfig;

    const META: Meta;

    /// Convert a partial configuration into a full configuration, with all values populated.
    fn from_partial(partial: Self::Partial) -> Self;
}

/// Represents an enumerable setting for use within a [`Config`].
pub trait ConfigEnum: Sized + Schematic {
    const META: Meta;

    /// Return a list of all variants for the enum. Only unit variants are supported.
    fn variants() -> Vec<Self>;
}

derive_enum!(
    /// Represents an extendable setting, either a string or a list of strings.
    #[serde(untagged)]
    pub enum ExtendsFrom {
        String(String),
        List(Vec<String>),
    }
);

impl Default for ExtendsFrom {
    fn default() -> Self {
        Self::List(vec![])
    }
}

impl Schematic for ExtendsFrom {
    fn generate_schema() -> schematic_types::SchemaType {
        use schematic_types::*;

        let mut schema = SchemaType::union([
            SchemaType::string(),
            SchemaType::array(SchemaType::string()),
        ]);
        schema.set_name("ExtendsFrom");
        schema
    }
}
