use crate::error::ConfigError;
use std::str::FromStr;

/// Parse a string into a boolean. Will parse `1`, `true`, `yes`, `on`,
/// and `enabled` as true, and everything else as false.
pub fn parse_bool(var: String) -> Result<bool, ConfigError> {
    match var.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" | "enable" => Ok(true),
        _ => Ok(false),
    }
}

fn split<T: FromStr>(var: String, delim: char) -> Result<Vec<T>, ConfigError> {
    let mut list = vec![];

    for s in var.split(delim) {
        let value: T = s.trim().parse().map_err(|_| {
            ConfigError::Message(format!("Failed to parse \"{s}\" into the correct type."))
        })?;

        list.push(value);
    }

    Ok(list)
}

/// Split a variable on each comma (`,`) and parse into a list of values.
pub fn split_comma<T: FromStr>(var: String) -> Result<Vec<T>, ConfigError> {
    split(var, ',')
}

/// Split a variable on each colon (`:`) and parse into a list of values.
pub fn split_colon<T: FromStr>(var: String) -> Result<Vec<T>, ConfigError> {
    split(var, ':')
}

/// Split a variable on each semicolon (`;`) and parse into a list of values.
pub fn split_semicolon<T: FromStr>(var: String) -> Result<Vec<T>, ConfigError> {
    split(var, ';')
}

/// Split a variable on each space (` `) and parse into a list of values.
pub fn split_space<T: FromStr>(var: String) -> Result<Vec<T>, ConfigError> {
    split(var, ' ')
}
