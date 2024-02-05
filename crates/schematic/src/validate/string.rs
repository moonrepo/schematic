use super::{map_err, ValidateError, Validator};
pub use garde::rules::{
    alphanumeric::Alphanumeric, ascii::Ascii, contains::Contains, length::HasSimpleLength,
    pattern::Pattern,
};

/// Validate a string is only composed of alpha-numeric characters.
pub fn alphanumeric<T: Alphanumeric, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    garde::rules::alphanumeric::apply(value, ()).map_err(map_err)
}

/// Validate a string is only composed of ASCII characters.
pub fn ascii<T: Ascii, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    garde::rules::ascii::apply(value, ()).map_err(map_err)
}

/// Validate a string contains the provided pattern.
pub fn contains<T: Contains, D, C>(pattern: &str) -> Validator<T, D, C> {
    let pattern = pattern.to_owned();

    Box::new(move |value, _, _| garde::rules::contains::apply(value, (&pattern,)).map_err(map_err))
}

/// Validate a string matches the provided regex pattern.
pub fn regex<T: Pattern, D, C>(pattern: &str) -> Validator<T, D, C> {
    let pattern = garde::rules::pattern::regex::Regex::new(pattern).unwrap();

    Box::new(move |value, _, _| garde::rules::pattern::apply(value, (&pattern,)).map_err(map_err))
}

/// Validate the value is not empty.
pub fn not_empty<T: HasSimpleLength, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    if value.length() == 0 {
        return Err(ValidateError::new("must not be empty"));
    }

    Ok(())
}
