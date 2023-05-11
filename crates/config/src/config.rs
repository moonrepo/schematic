use serde::de::DeserializeOwned;

pub trait PartialConfig: DeserializeOwned + Sized {
    fn default_values() -> Self;
}

pub trait Config: Sized {
    type Partial: PartialConfig;
}
