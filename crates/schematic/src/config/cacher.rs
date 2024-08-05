use super::error::HandlerError;
use std::collections::HashMap;
use std::path::PathBuf;

/// A system for reading and writing to a cache for URL based configurations.
pub trait Cacher {
    /// If the content was cached to the local file system, return the absolute path.
    fn get_file_path(&self, _url: &str) -> Result<Option<PathBuf>, HandlerError> {
        Ok(None)
    }

    /// Read content from the cache store.
    fn read(&mut self, url: &str) -> Result<Option<String>, HandlerError>;

    /// Write the provided content to the cache store.
    fn write(&mut self, url: &str, content: &str) -> Result<(), HandlerError>;
}

pub type BoxedCacher = Box<dyn Cacher>;

#[derive(Default)]
#[doc(hidden)]
pub struct MemoryCache {
    cache: HashMap<String, String>,
}

impl Cacher for MemoryCache {
    fn read(&mut self, url: &str) -> Result<Option<String>, HandlerError> {
        Ok(self.cache.get(url).map(|v| v.to_owned()))
    }

    fn write(&mut self, url: &str, content: &str) -> Result<(), HandlerError> {
        self.cache.insert(url.to_owned(), content.to_owned());

        Ok(())
    }
}
