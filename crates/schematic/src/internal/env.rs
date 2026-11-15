use super::parse_value;
use crate::config::{ConfigError, HandlerError, ParseEnvResult, PartialConfig};
use std::str::FromStr;

/// The environment variables of a nested config, viewing one that's missing
/// as its [`Default`], so that a variable can provide a config that no source
/// did. A parent's `#[setting(nested, env_prefix)]` is passed as `prefix`.
pub fn nested_env_values<T: PartialConfig>(
    value: Option<&T>,
    prefix: Option<&str>,
) -> Result<Option<T>, ConfigError> {
    let read = |value: &T| match prefix {
        Some(_) => value.env_values_with_prefix(prefix),
        None => value.env_values(),
    };

    match value {
        Some(value) => read(value),
        None => read(&T::default()),
    }
}

/// Finalize a config within a tuple variant that holds several values, whose
/// environment variables no layer holds, as the enum's would have to restate
/// the values beside it. Its defaults are part of the enum's own.
pub fn finalize_with_env<T: PartialConfig>(
    mut partial: T,
    context: &T::Context,
) -> Result<T, ConfigError> {
    if let Some(env) = partial.env_values()? {
        partial.merge(context, env)?;
    }

    partial.finalize(context)
}

pub struct EnvManager {
    found: bool,
    /// Prefixes to apply to derived keys, in order of precedence. The
    /// container's own prefix comes first, then the one a parent passed
    /// in with `#[setting(nested, env_prefix)]`, so that the key a config
    /// reports for itself is the one that wins.
    prefixes: Vec<String>,
}

impl EnvManager {
    pub fn new<T: AsRef<str>>(override_prefix: Option<T>, prefix: Option<T>) -> Self {
        Self {
            found: false,
            prefixes: vec![prefix, override_prefix]
                .into_iter()
                .flatten()
                .map(|pre| pre.as_ref().to_string())
                .filter(|pre| !pre.is_empty())
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.found
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
                        self.found = true;
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
            self.found = true;
        }

        Ok(partial)
    }
}
