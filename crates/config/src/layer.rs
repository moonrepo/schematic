use crate::config::Config;
use crate::source::Source;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Layer<T: Config> {
    pub partial: T::Partial,
    pub source: Source,
}
