//! Helpers for the code that `#[derive(Config)]`, `#[derive(ConfigEnum)]`,
//! and `#[derive(Schematic)]` generate, and for the loader. Nothing here is
//! public API, and each helper is reached through the module of its domain,
//! never re-exported here.

/// Parsing a `ConfigEnum` from a string.
#[cfg(feature = "config")]
pub mod config_enum;

/// Converting `#[setting(default)]` values into a setting's type.
#[cfg(feature = "config")]
pub mod defaults;

/// Reading environment variables into a partial.
#[cfg(all(feature = "config", feature = "env"))]
pub mod env;

/// Deserializing flattened settings.
#[cfg(feature = "config")]
pub mod flatten;

/// Applying the default and environment layers of a config.
#[cfg(feature = "config")]
pub mod layers;

/// Merging the settings of a partial.
#[cfg(feature = "config")]
pub mod merge;

/// Building the schemas of configs, their partials, and tagged variants.
pub mod schema;

/// Collecting the settings of nested configs.
#[cfg(feature = "config")]
pub mod settings;

/// Running the `#[setting(transform)]` functions of a partial.
#[cfg(feature = "config")]
pub mod transform;

/// Deserializing untagged enums and literal unions.
#[cfg(feature = "config")]
pub mod untagged;

/// Validating the settings of a partial.
#[cfg(all(feature = "config", feature = "validate"))]
pub mod validate;
