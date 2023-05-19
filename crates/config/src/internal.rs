use crate::error::ConfigError;
use std::{env, str::FromStr};

pub fn default_from_env_var<T: FromStr>(key: &str) -> Result<Option<T>, ConfigError> {
    parse_from_env_var(key, |var| {
        var.parse::<T>()
            .map_err(|_| ConfigError::Message("Failed to parse into the correct type.".into()))
    })
}

pub fn parse_from_env_var<T>(
    key: &str,
    parser: impl Fn(String) -> Result<T, ConfigError>,
) -> Result<Option<T>, ConfigError> {
    if let Ok(var) = env::var(key) {
        let value =
            parser(var).map_err(|e| ConfigError::InvalidEnvVar(key.to_owned(), e.to_string()))?;

        return Ok(Some(value));
    }

    Ok(None)
}

pub fn merge_settings<T, C>(prev: Option<T>, next: Option<T>, _: &C) -> Option<T> {
    if next.is_some() {
        next
    } else {
        prev
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn merge_settings_with_func<T, C>(
    prev: Option<T>,
    next: Option<T>,
    context: &C,
    merger: impl Fn(T, T, &C) -> Option<T>,
) -> Option<T> {
    if prev.is_some() && next.is_some() {
        merger(prev.unwrap(), next.unwrap(), context)
    } else if next.is_some() {
        next
    } else {
        prev
    }
}
