use super::parse_value;
use crate::config::{HandlerError, ParseEnvResult};
use std::str::FromStr;

pub struct EnvManager {
    count: u8,
    prefix: String,
}

impl EnvManager {
    pub fn new<T: AsRef<str>>(prefix: Option<T>) -> Self {
        Self {
            count: 0,
            prefix: prefix
                .map(|pre| pre.as_ref().to_string())
                .unwrap_or_default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get<T: FromStr>(&mut self, key: &str) -> ParseEnvResult<T> {
        self.get_and_parse(key, |value| parse_value(value).map(|v| Some(v)))
    }

    pub fn get_and_parse<T>(
        &mut self,
        key: &str,
        parser: impl Fn(String) -> ParseEnvResult<T>,
    ) -> ParseEnvResult<T> {
        let key = format!("{}{key}", self.prefix);

        if let Ok(value) = std::env::var(&key) {
            return parser(value)
                .inspect(|inner| {
                    if inner.is_some() {
                        self.count += 1;
                    }
                })
                .map_err(|error| {
                    HandlerError(format!("Invalid environment variable {key}: {error}"))
                });
        }

        Ok(None)
    }

    pub fn nested<T>(&mut self, partial: Option<T>) -> ParseEnvResult<T> {
        if partial.is_some() {
            self.count += 1;
        }

        Ok(partial)
    }
}
