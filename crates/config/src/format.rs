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

impl Format {
    pub fn is_json(&self) -> bool {
        #[cfg(feature = "json")]
        {
            matches!(self, Format::Json)
        }
        #[cfg(not(feature = "json"))]
        {
            false
        }
    }

    pub fn is_toml(&self) -> bool {
        #[cfg(feature = "toml")]
        {
            matches!(self, Format::Toml)
        }
        #[cfg(not(feature = "toml"))]
        {
            false
        }
    }

    pub fn is_yaml(&self) -> bool {
        #[cfg(feature = "yaml")]
        {
            matches!(self, Format::Yaml)
        }
        #[cfg(not(feature = "yaml"))]
        {
            false
        }
    }
}
