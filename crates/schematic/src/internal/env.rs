use crate::config::{HandlerError, ParseEnvResult};
use crate::helpers::short_type_name;
use std::fmt::Display;
use std::str::FromStr;

/// Parse the value of an environment variable into a setting's type, naming
/// the type and why it failed, like `number too large to fit in target type`.
pub fn parse_value<T, V>(value: V) -> Result<T, HandlerError>
where
    T: FromStr,
    T::Err: Display,
    V: AsRef<str>,
{
    let value = value.as_ref();

    value.parse::<T>().map_err(|error| {
        HandlerError(format!(
            "Failed to parse \"{value}\" into {}: {error}.",
            short_type_name::<T>()
        ))
    })
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

    /// Return the layer the variables were read into, or `None` when none was
    /// set, so a layer is only reported when it sets something.
    pub fn finish<T>(self, layer: T) -> Option<T> {
        self.found.then_some(layer)
    }

    /// Pick a variant of a tagged enum from the variable its tag derives,
    /// using `parse` to match the value by a variant's name or alias, and
    /// listing `expected` in the error when it matches none. A variable naming
    /// the variant `current` already holds picks nothing, so what the sources
    /// gave it stands.
    pub fn pick_variant<T>(
        &mut self,
        key: &str,
        current: &T,
        expected: &[&str],
        parse: impl Fn(&str) -> Option<T>,
    ) -> ParseEnvResult<T> {
        let picked = self.get_and_parse_prefixed(key, |value| match parse(&value) {
            Some(variant) => Ok(Some(variant)),
            None => Err(HandlerError::new(format!(
                "Unknown variant `{value}`, expected one of {}.",
                expected
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        })?;

        Ok(picked
            .filter(|picked| std::mem::discriminant(picked) != std::mem::discriminant(current)))
    }

    /// Get a variable using the exact key, ignoring the prefix.
    /// For explicit keys defined with `#[setting(env)]`.
    pub fn get<T>(&mut self, key: &str) -> ParseEnvResult<T>
    where
        T: FromStr,
        T::Err: Display,
    {
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
    pub fn get_prefixed<T>(&mut self, key: &str) -> ParseEnvResult<T>
    where
        T: FromStr,
        T::Err: Display,
    {
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
