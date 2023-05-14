use crate::error::ConfigError;
use crate::validator::{SettingPath, ValidatorError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait PartialConfig: Default + DeserializeOwned + Sized {
    fn default_values() -> Result<Self, ConfigError>;

    fn extends_from(&self) -> Option<ExtendsFrom>;

    fn merge(&mut self, next: Self);
}

pub trait Config: Sized {
    type Partial: PartialConfig;

    fn default_values() -> Result<Self, ConfigError> {
        Ok(Self::from_partial(
            <Self::Partial as PartialConfig>::default_values()?,
        ))
    }

    fn from_partial(partial: Self::Partial) -> Self;

    fn partial() -> Self::Partial {
        <Self::Partial as Default>::default()
    }

    fn validate(&self) -> Result<(), ValidatorError> {
        self.validate_with_path(SettingPath::default())
    }

    fn validate_with_path(&self, _path: SettingPath) -> Result<(), ValidatorError> {
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
