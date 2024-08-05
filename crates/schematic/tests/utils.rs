#![allow(dead_code)]

use schematic::{Cacher, HandlerError};
use std::path::PathBuf;
use std::{env, fs};

pub fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/__fixtures__")
        .join(name)
}

pub struct SandboxCacher {
    pub root: PathBuf,
}

impl Cacher for SandboxCacher {
    fn get_file_path(&self, url: &str) -> Result<Option<PathBuf>, HandlerError> {
        let name = &url[url.rfind('/').unwrap() + 1..];

        Ok(Some(self.root.join(name)))
    }

    /// Read content from the cache store.
    fn read(&mut self, url: &str) -> Result<Option<String>, HandlerError> {
        self.get_file_path(url).map(|path| {
            if path.as_ref().is_some_and(|p| p.exists()) {
                fs::read_to_string(path.unwrap()).ok()
            } else {
                None
            }
        })
    }

    /// Write the provided content to the cache store.
    fn write(&mut self, url: &str, content: &str) -> Result<(), HandlerError> {
        fs::write(self.get_file_path(url).unwrap().unwrap(), content).unwrap();

        Ok(())
    }
}
