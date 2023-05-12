use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait PartialConfig: Default + DeserializeOwned + Sized {
    fn default_values() -> Self;

    fn extends_from(&self) -> Option<ExtendsFrom>;

    fn merge(&mut self, next: Self);
}

pub trait Config: Sized {
    type Partial: PartialConfig;

    fn from_defaults() -> Self {
        Self::from_partial(<Self::Partial as PartialConfig>::default_values())
    }

    fn from_partial(partial: Self::Partial) -> Self;
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
