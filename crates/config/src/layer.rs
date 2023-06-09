use crate::config::Config;
use crate::source::Source;
use serde::Serialize;

/// A layer of configuration that was loaded, and used to create the final state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Layer<T: Config> {
    /// The partial configuration that was loaded.
    pub partial: T::Partial,

    /// The source location of the partial.
    pub source: Source,
}
