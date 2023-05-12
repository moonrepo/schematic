use crate::error::ConfigError;
use std::{env, str::FromStr};

pub fn default_from_env_var<T: FromStr>(
    key: &str,
    fallback: Option<T>,
) -> Result<Option<T>, ConfigError> {
    parse_from_env_var(
        key,
        |var| {
            var.parse::<T>()
                .map_err(|_| ConfigError::Message("Failed to parse into the correct type.".into()))
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
        let value =
            parser(var).map_err(|e| ConfigError::InvalidEnvVar(key.to_owned(), e.to_string()))?;

        return Ok(Some(value));
    }

    Ok(fallback)
}
