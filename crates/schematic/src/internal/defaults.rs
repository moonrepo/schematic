use crate::config::ConfigError;
use std::str::FromStr;

/// Converts a string literal default into the setting's type, through
/// `TryFrom<&str>` when it's implemented, and `FromStr` otherwise, so that
/// types like `IpAddr` that only parse can take a string default. Which one
/// applies is picked at compile time, by calling `parse_default` on
/// `&&DefaultString<T>`, where the `TryFrom` impl matches first.
pub struct DefaultString<T>(std::marker::PhantomData<T>);

impl<T> DefaultString<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

pub trait DefaultStringViaTryFrom<T> {
    fn parse_default(&self, path: &str, value: &'static str) -> Result<T, ConfigError>;
}

impl<T> DefaultStringViaTryFrom<T> for &DefaultString<T>
where
    T: TryFrom<&'static str>,
    <T as TryFrom<&'static str>>::Error: std::fmt::Display,
{
    fn parse_default(&self, path: &str, value: &'static str) -> Result<T, ConfigError> {
        T::try_from(value).map_err(|error| invalid_default(path, error))
    }
}

pub trait DefaultStringViaFromStr<T> {
    fn parse_default(&self, path: &str, value: &'static str) -> Result<T, ConfigError>;
}

impl<T> DefaultStringViaFromStr<T> for DefaultString<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn parse_default(&self, path: &str, value: &'static str) -> Result<T, ConfigError> {
        value
            .parse()
            .map_err(|error: T::Err| invalid_default(path, error))
    }
}

/// Convert the error of a `#[setting(default)]` handler function for the
/// setting at `path`.
pub fn handle_default_result<T, E: std::error::Error>(
    path: &str,
    result: Result<T, E>,
) -> Result<T, ConfigError> {
    result.map_err(|error| invalid_default(path, error))
}

fn invalid_default(path: &str, error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidDefaultValue {
        path: path.into(),
        error: error.to_string(),
    }
}
