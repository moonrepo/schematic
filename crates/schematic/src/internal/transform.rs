use crate::config::ConfigError;
use std::fmt::Display;

/// Convert the error of a `#[setting(transform)]` function for the setting
/// at `path`. It's documented to return a `TransformResult`, while any error
/// that displays is accepted, as each one that converted into a
/// `ConfigError` was before.
pub fn handle_transform_result<T, E: Display>(
    path: &str,
    result: Result<T, E>,
) -> Result<T, ConfigError> {
    result.map_err(|error| ConfigError::TransformFailed {
        path: path.into(),
        error: error.to_string(),
    })
}
