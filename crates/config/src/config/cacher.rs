use crate::config::errors::ConfigError;
use std::collections::HashMap;

/// A system for reading and writing to a cache for URL based configurations.
pub trait Cacher {
    /// Read content from the cache store.
    fn read(&mut self, url: &str) -> Result<Option<String>, ConfigError>;

    /// Write the provided content to the cache store.
    fn write(&mut self, url: &str, content: &str) -> Result<(), ConfigError>;
}

pub type BoxedCacher = Box<dyn Cacher>;

#[derive(Default)]
#[doc(hidden)]
pub struct MemoryCache {
    cache: HashMap<String, String>,
}

impl Cacher for MemoryCache {
    fn read(&mut self, url: &str) -> Result<Option<String>, ConfigError> {
        Ok(self.cache.get(url).map(|v| v.to_owned()))
    }

    fn write(&mut self, url: &str, content: &str) -> Result<(), ConfigError> {
        self.cache.insert(url.to_owned(), content.to_owned());

        Ok(())
    }
}
