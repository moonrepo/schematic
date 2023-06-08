use crate::errors::ConfigError;
use std::str::FromStr;

fn parse<T: FromStr, V: AsRef<str>>(value: V) -> Result<T, ConfigError> {
    let value = value.as_ref();

    value.parse::<T>().map_err(|_| {
        ConfigError::Message(format!(
            "Failed to parse \"{value}\" into the correct type."
        ))
    })
}

/// Ignore the environment variable if it's empty and fallback to the previous or default value.
pub fn ignore_empty<T: FromStr>(var: String) -> Result<Option<T>, ConfigError> {
    let var = var.trim();

    if var.is_empty() {
        return Ok(None);
    }

    parse(var).map(|v| Some(v))
}

/// Parse a string into a boolean. Will parse `1`, `true`, `yes`, `on`,
/// and `enabled` as true, and everything else as false.
pub fn parse_bool(var: String) -> Result<Option<bool>, ConfigError> {
    Ok(match var.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" | "enable" => Some(true),
        _ => Some(false),
    })
}

fn split<T: FromStr>(var: String, delim: char) -> Result<Option<Vec<T>>, ConfigError> {
    let mut list = vec![];

    for s in var.split(delim) {
        let value = s.trim();

        if !value.is_empty() {
            list.push(parse(value)?);
        }
    }

    Ok(Some(list))
}

/// Split a variable on each comma (`,`) and parse into a list of values.
pub fn split_comma<T: FromStr>(var: String) -> Result<Option<Vec<T>>, ConfigError> {
    split(var, ',')
}

/// Split a variable on each colon (`:`) and parse into a list of values.
pub fn split_colon<T: FromStr>(var: String) -> Result<Option<Vec<T>>, ConfigError> {
    split(var, ':')
}

/// Split a variable on each semicolon (`;`) and parse into a list of values.
pub fn split_semicolon<T: FromStr>(var: String) -> Result<Option<Vec<T>>, ConfigError> {
    split(var, ';')
}

/// Split a variable on each space (` `) and parse into a list of values.
pub fn split_space<T: FromStr>(var: String) -> Result<Option<Vec<T>>, ConfigError> {
    split(var, ' ')
}
