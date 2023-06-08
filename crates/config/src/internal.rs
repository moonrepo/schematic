use crate::config::PartialConfig;
use crate::errors::ConfigError;
use crate::merge::merge_partial;
use std::{env, str::FromStr};

pub fn default_from_env_var<T: FromStr>(key: &str) -> Result<Option<T>, ConfigError> {
    parse_from_env_var(key, |var| {
        var.parse::<T>()
            .map(|v| Some(v))
            .map_err(|_| ConfigError::Message("Failed to parse into the correct type.".into()))
    })
}

pub fn parse_from_env_var<T>(
    key: &str,
    parser: impl Fn(String) -> Result<Option<T>, ConfigError>,
) -> Result<Option<T>, ConfigError> {
    if let Ok(var) = env::var(key) {
        let value =
            parser(var).map_err(|e| ConfigError::InvalidEnvVar(key.to_owned(), e.to_string()))?;

        return Ok(value);
    }

    Ok(None)
}

#[allow(clippy::unnecessary_unwrap)]
pub fn merge_setting<T, C>(
    prev: Option<T>,
    next: Option<T>,
    context: &C,
    merger: impl Fn(T, T, &C) -> Result<Option<T>, ConfigError>,
) -> Result<Option<T>, ConfigError> {
    if prev.is_some() && next.is_some() {
        merger(prev.unwrap(), next.unwrap(), context)
    } else if next.is_some() {
        Ok(next)
    } else {
        Ok(prev)
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn merge_partial_setting<T: PartialConfig>(
    prev: Option<T>,
    next: Option<T>,
    context: &T::Context,
) -> Result<Option<T>, ConfigError> {
    if prev.is_some() && next.is_some() {
        merge_partial(prev.unwrap(), next.unwrap(), context)
    } else if next.is_some() {
        // For optional nested configs, defaults values are not provided by the
        // `default_values` method since we return `None`. However, if a value
        // is provided during the merging process, then we need to lazily load
        // the default values to merge against.
        merge_partial(T::default_values(context)?, next.unwrap(), context)
    } else {
        Ok(prev)
    }
}
