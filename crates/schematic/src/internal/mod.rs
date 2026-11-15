//! Helpers for the code that `#[derive(Config)]` and `#[derive(ConfigEnum)]`
//! generate, and for the loader. Nothing here is public API, and each helper
//! is reached through the module of its domain, never re-exported here.

/// Normalizing input for `#[config(before_parse)]`.
pub mod casing;

/// Converting `#[setting(default)]` values into a setting's type.
pub mod defaults;

/// Reading environment variables into a partial.
#[cfg(feature = "env")]
pub mod env;

/// Applying the default and environment layers of a config.
pub mod layers;

/// Merging the settings of a partial.
pub mod merge;

/// Turning a config's schema into its partial's.
pub mod schema;

/// Collecting the settings of nested configs.
pub mod settings;

/// Deserializing untagged enums and literal unions.
pub mod untagged;

/// Validating the settings of a partial.
#[cfg(feature = "validate")]
pub mod validate;
