use crate::config::source::Source;
use crate::config::Config;
use serde::{Deserialize, Serialize};

/// A layer of configuration that was loaded, and used to create the final state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Layer<T: Config> {
    /// The partial configuration that was loaded.
    pub partial: T::Partial,

    /// The source location of the partial.
    pub source: Source,
}
