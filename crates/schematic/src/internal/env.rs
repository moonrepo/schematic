use super::parse_value;
use crate::config::{HandlerError, ParseEnvResult};
use std::str::FromStr;

pub struct EnvManager {
    count: u8,
    /// Prefixes to apply to derived keys, in order of precedence. The
    /// container's own prefix comes first, then the one a parent passed
    /// in with `#[setting(nested, env_prefix)]`. The container's own must
    /// win, because `finalize` re-reads the environment without a parent
    /// in the picture, and that read is applied last.
    prefixes: Vec<String>,
}

impl EnvManager {
    pub fn new<T: AsRef<str>>(override_prefix: Option<T>, prefix: Option<T>) -> Self {
        Self {
            count: 0,
            prefixes: vec![override_prefix, prefix]
                .into_iter()
                .flatten()
                .map(|pre| pre.as_ref().to_string())
                .filter(|pre| !pre.is_empty())
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get a variable using the exact key, ignoring the prefix.
    /// For explicit keys defined with `#[setting(env)]`.
    pub fn get<T: FromStr>(&mut self, key: &str) -> ParseEnvResult<T> {
        self.get_and_parse(key, |value| parse_value(value).map(|v| Some(v)))
    }

    /// Get and parse a variable using the exact key, ignoring the prefix.
    /// For explicit keys defined with `#[setting(env)]`.
    pub fn get_and_parse<T>(
        &mut self,
        key: &str,
        parser: impl Fn(String) -> ParseEnvResult<T>,
    ) -> ParseEnvResult<T> {
        self.read(key, parser)
    }

    /// Get a variable using the key with a prefix applied.
    /// For keys derived from setting names when using `env_prefix`.
    pub fn get_prefixed<T: FromStr>(&mut self, key: &str) -> ParseEnvResult<T> {
        self.get_and_parse_prefixed(key, |value| parse_value(value).map(|v| Some(v)))
    }

    /// Get and parse a variable using the key with a prefix applied. Each
    /// prefix is tried in order, and the first variable found wins.
    /// For keys derived from setting names when using `env_prefix`.
    pub fn get_and_parse_prefixed<T>(
        &mut self,
        key: &str,
        parser: impl Fn(String) -> ParseEnvResult<T>,
    ) -> ParseEnvResult<T> {
        for index in 0..self.prefixes.len() {
            let full_key = format!("{}{key}", self.prefixes[index]);

            if let Some(value) = self.read(&full_key, &parser)? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    fn read<T>(
        &mut self,
        key: &str,
        parser: impl Fn(String) -> ParseEnvResult<T>,
    ) -> ParseEnvResult<T> {
        if let Ok(value) = std::env::var(key) {
            return parser(value)
                .inspect(|inner| {
                    if inner.is_some() {
                        self.count += 1;
                    }
                })
                .map_err(|error| {
                    HandlerError(format!("Invalid environment variable {key}. {error}"))
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
