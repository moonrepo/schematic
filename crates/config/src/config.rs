use serde::de::DeserializeOwned;

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

pub enum ExtendsFrom<'partial> {
    String(&'partial String),
    List(&'partial Vec<String>),
}
