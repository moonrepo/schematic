use crate::config::errors::ConfigError;
use crate::config::format::Format;

/// A system for reading and writing to a cache for URL based configurations.
pub trait Cacher {
    /// Read content from the cache store for the provided URL and format.
    fn read(&self, _url: &str, _format: &Format) -> Result<Option<String>, ConfigError> {
        Ok(None)
    }

    /// Write the provided content to the cache store.
    fn write(&self, _content: &str) -> Result<(), ConfigError> {
        Ok(())
    }
}

pub type BoxedCacher = Box<dyn Cacher>;
