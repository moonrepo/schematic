use crate::{HandlerError, ParseEnvResult, internal::env::parse_value};
use std::fmt::Display;
use std::str::FromStr;

/// Ignore the environment variable if it's empty and fallback to the previous or default value.
pub fn ignore_empty<T>(var: String) -> ParseEnvResult<T>
where
    T: FromStr,
    T::Err: Display,
{
    let var = var.trim();

    if var.is_empty() {
        return Ok(None);
    }

    parse_value(var).map(|v| Some(v))
}

/// Parse a string into a boolean, ignoring case and surrounding whitespace.
/// Will parse `1`, `true`, `yes`, `on`, `enable`, and `enabled` as true, and
/// `0`, `false`, `no`, `off`, `disable`, and `disabled` as false. An empty
/// value is ignored, like an unset variable, and anything else is an error.
pub fn parse_bool(var: String) -> ParseEnvResult<bool> {
    match var.trim().to_lowercase().as_str() {
        "" => Ok(None),
        "1" | "true" | "yes" | "on" | "enabled" | "enable" => Ok(Some(true)),
        "0" | "false" | "no" | "off" | "disabled" | "disable" => Ok(Some(false)),
        _ => Err(HandlerError::new(format!(
            "Failed to parse \"{var}\" into bool, expected one of 1, true, yes, on, enable, or enabled, or 0, false, no, off, disable, or disabled."
        ))),
    }
}

fn split<T>(var: String, delim: char) -> ParseEnvResult<Vec<T>>
where
    T: FromStr,
    T::Err: Display,
{
    let mut list = vec![];

    for s in var.split(delim) {
        let value = s.trim();

        if !value.is_empty() {
            list.push(parse_value(value)?);
        }
    }

    Ok(Some(list))
}

/// Split a variable on each comma (`,`) and parse into a list of values.
pub fn split_comma<T>(var: String) -> ParseEnvResult<Vec<T>>
where
    T: FromStr,
    T::Err: Display,
{
    split(var, ',')
}

/// Split a variable on each colon (`:`) and parse into a list of values.
pub fn split_colon<T>(var: String) -> ParseEnvResult<Vec<T>>
where
    T: FromStr,
    T::Err: Display,
{
    split(var, ':')
}

/// Split a variable on each semicolon (`;`) and parse into a list of values.
pub fn split_semicolon<T>(var: String) -> ParseEnvResult<Vec<T>>
where
    T: FromStr,
    T::Err: Display,
{
    split(var, ';')
}

/// Split a variable on each space (` `) and parse into a list of values.
pub fn split_space<T>(var: String) -> ParseEnvResult<Vec<T>>
where
    T: FromStr,
    T::Err: Display,
{
    split(var, ' ')
}
