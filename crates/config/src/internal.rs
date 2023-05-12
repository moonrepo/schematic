use crate::error::ConfigError;
use std::{env, str::FromStr};

pub fn default_from_env_var<T: FromStr>(
    key: &str,
    fallback: Option<T>,
) -> Result<Option<T>, ConfigError> {
    parse_from_env_var(
        key,
        |var| {
            var.parse::<T>().map_err(|_| {
                ConfigError::InvalidEnvVar(
                    key.to_owned(),
                    "Failed to parse into the correct value.".into(),
                )
            })
        },
        fallback,
    )
}

pub fn parse_from_env_var<T>(
    key: &str,
    parser: impl Fn(String) -> Result<T, ConfigError>,
    fallback: Option<T>,
) -> Result<Option<T>, ConfigError> {
    if let Ok(var) = env::var(key) {
        return Ok(Some(parser(var)?));
    }

    Ok(fallback)
}
