use crate::error::ConfigError;
use crate::validator::{SettingPath, ValidatorError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct ConfigMeta {
    /// Name of the struct.
    pub name: &'static str,

    /// File name of the loaded config.
    pub file: Option<&'static str>,
}

pub trait PartialConfig: Clone + Default + DeserializeOwned + Serialize + Sized {
    type Context: Default;

    fn default_values(context: &Self::Context) -> Result<Self, ConfigError>;

    fn env_values() -> Result<Self, ConfigError>;

    fn extends_from(&self) -> Option<ExtendsFrom>;

    fn merge(&mut self, context: &Self::Context, next: Self) -> Result<(), ConfigError>;
}

pub trait Config: Sized {
    type Partial: PartialConfig;

    const META: ConfigMeta;

    fn from_partial(
        context: &<Self::Partial as PartialConfig>::Context,
        partial: Self::Partial,
        with_env: bool,
    ) -> Result<Self, ConfigError>;

    fn validate(
        &self,
        context: &<Self::Partial as PartialConfig>::Context,
    ) -> Result<(), ValidatorError> {
        self.validate_with_path(context, SettingPath::default())
    }

    fn validate_with_path(
        &self,
        _context: &<Self::Partial as PartialConfig>::Context,
        _path: SettingPath,
    ) -> Result<(), ValidatorError> {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ExtendsFrom {
    String(String),
    List(Vec<String>),
}

impl Default for ExtendsFrom {
    fn default() -> Self {
        Self::List(vec![])
    }
}
