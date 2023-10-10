use crate::config::errors::ConfigError;

/// A system for reading and writing to a cache for URL based configurations.
pub trait Cacher {
    /// Read content from the cache store.
    fn read(&self, _url: &str) -> Result<Option<String>, ConfigError> {
        Ok(None)
    }

    /// Write the provided content to the cache store.
    fn write(&self, _url: &str, _content: &str) -> Result<(), ConfigError> {
        Ok(())
    }
}

pub type BoxedCacher = Box<dyn Cacher>;

#[doc(hidden)]
pub struct NoCache;

impl Cacher for NoCache {}
