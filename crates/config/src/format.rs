use serde::{Deserialize, Serialize};

/// Supported source configuration formats.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    // This is to simply handle the use case when no features are
    // enabled. If this doesn't exist, Rust errors with no variants.
    #[doc(hidden)]
    #[default]
    None,

    #[cfg(feature = "json")]
    Json,

    #[cfg(feature = "toml")]
    Toml,

    #[cfg(feature = "yaml")]
    Yaml,
}
