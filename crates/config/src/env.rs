use crate::error::ConfigError;

fn split(var: String, delim: char) -> Result<Vec<String>, ConfigError> {
    Ok(var.split(delim).map(|s| s.to_owned()).collect())
}

/// Split a variable on each comma (,) into a list of values.
pub fn split_comma(var: String) -> Result<Vec<String>, ConfigError> {
    split(var, ',')
}

/// Split a variable on each colon (:) into a list of values.
pub fn split_colon(var: String) -> Result<Vec<String>, ConfigError> {
    split(var, ':')
}

/// Split a variable on each semicolon (;) into a list of values.
pub fn split_semicolon(var: String) -> Result<Vec<String>, ConfigError> {
    split(var, ';')
}

/// Split a variable on each space ( ) into a list of values.
pub fn split_space(var: String) -> Result<Vec<String>, ConfigError> {
    split(var, ' ')
}
