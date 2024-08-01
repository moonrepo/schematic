use super::error::ConfigError;
use async_trait::async_trait;
use std::collections::HashMap;

/// A system for reading and writing to a cache for URL based configurations.
#[async_trait]
pub trait Cacher {
    /// Read content from the cache store.
    async fn read(&mut self, url: &str) -> Result<Option<String>, ConfigError>;

    /// Write the provided content to the cache store.
    async fn write(&mut self, url: &str, content: &str) -> Result<(), ConfigError>;
}

pub type BoxedCacher = Box<dyn Cacher>;

#[derive(Default)]
#[doc(hidden)]
pub struct MemoryCache {
    cache: HashMap<String, String>,
}

#[async_trait]
impl Cacher for MemoryCache {
    async fn read(&mut self, url: &str) -> Result<Option<String>, ConfigError> {
        Ok(self.cache.get(url).map(|v| v.to_owned()))
    }

    async fn write(&mut self, url: &str, content: &str) -> Result<(), ConfigError> {
        self.cache.insert(url.to_owned(), content.to_owned());

        Ok(())
    }
}
