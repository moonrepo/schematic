use crate::error::ConfigError;
use crate::validator::{SettingPath, ValidatorError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct ConfigMeta {
    /// Name of the struct.
    pub name: &'static str,

    /// File name of the loaded config.
    pub file: Option<&'static str>,
}

pub trait PartialConfig: Default + DeserializeOwned + Sized {
    type Context: Default;

    fn default_values(context: &Self::Context) -> Result<Self, ConfigError>;

    fn extends_from(&self) -> Option<ExtendsFrom>;

    fn merge(&mut self, next: Self);
}

pub trait Config: Sized {
    type Partial: PartialConfig;

    const META: ConfigMeta;

    fn default_values(
        context: &<Self::Partial as PartialConfig>::Context,
    ) -> Result<Self, ConfigError> {
        Ok(Self::from_partial(
            <Self::Partial as PartialConfig>::default_values(context)?,
        ))
    }

    fn from_partial(partial: Self::Partial) -> Self;

    fn partial() -> Self::Partial {
        <Self::Partial as Default>::default()
    }

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
