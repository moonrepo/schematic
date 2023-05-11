use crate::config::Config;
use crate::error::ConfigError;
use crate::source::{Source, SourceFormat};
use std::path::PathBuf;

pub struct ConfigLoader<T: Config> {
    marker: std::marker::PhantomData<T>,
    source_format: SourceFormat,
    sources: Vec<Source>,
}

impl<T: Config> ConfigLoader<T> {
    pub fn new(source_format: SourceFormat) -> Self {
        Self {
            marker: std::marker::PhantomData,
            source_format,
            sources: vec![],
        }
    }

    pub fn code<P: TryInto<String>>(&mut self, code: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::code(code)?);

        Ok(self)
    }

    pub fn file<P: TryInto<PathBuf>>(&mut self, path: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::file(path)?);

        Ok(self)
    }

    pub fn url<P: TryInto<String>>(&mut self, url: P) -> Result<&mut Self, ConfigError> {
        self.sources.push(Source::url(url)?);

        Ok(self)
    }

    // pub fn load() -> T {
    //     T::default()
    // }
}

// fn test() {
//     let config: SomeConfig = ConfigLoader::new(SourceFormat::Yaml)
//         .file("foo.yml")?
//         .url("https://example.com/bar.yml")?
//         .load()?;
// }
