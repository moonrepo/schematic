use crate::{ParseEnvResult, internal};
use std::str::FromStr;

/// Ignore the environment variable if it's empty and fallback to the previous or default value.
pub fn ignore_empty<T: FromStr>(var: String) -> ParseEnvResult<T> {
    let var = var.trim();

    if var.is_empty() {
        return Ok(None);
    }

    internal::parse_value(var).map(|v| Some(v))
}

/// Parse a string into a boolean. Will parse `1`, `true`, `yes`, `on`,
/// and `enabled` as true, and everything else as false.
pub fn parse_bool(var: String) -> ParseEnvResult<bool> {
    Ok(match var.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" | "enable" => Some(true),
        _ => Some(false),
    })
}

fn split<T: FromStr>(var: String, delim: char) -> ParseEnvResult<Vec<T>> {
    let mut list = vec![];

    for s in var.split(delim) {
        let value = s.trim();

        if !value.is_empty() {
            list.push(internal::parse_value(value)?);
        }
    }

    Ok(Some(list))
}

/// Split a variable on each comma (`,`) and parse into a list of values.
pub fn split_comma<T: FromStr>(var: String) -> ParseEnvResult<Vec<T>> {
    split(var, ',')
}

/// Split a variable on each colon (`:`) and parse into a list of values.
pub fn split_colon<T: FromStr>(var: String) -> ParseEnvResult<Vec<T>> {
    split(var, ':')
}

/// Split a variable on each semicolon (`;`) and parse into a list of values.
pub fn split_semicolon<T: FromStr>(var: String) -> ParseEnvResult<Vec<T>> {
    split(var, ';')
}

/// Split a variable on each space (` `) and parse into a list of values.
pub fn split_space<T: FromStr>(var: String) -> ParseEnvResult<Vec<T>> {
    split(var, ' ')
}
