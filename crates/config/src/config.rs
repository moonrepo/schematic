use serde::de::DeserializeOwned;

pub trait PartialConfig: Default + DeserializeOwned + Sized {
    fn default_values() -> Self;
    fn merge(&mut self, next: Self);
}

pub trait Config: Sized {
    type Partial: PartialConfig;

    fn from_defaults() -> Self;
    fn from_partial(partial: Self::Partial) -> Self;
}
