use super::{map_err, ValidateError};
use crate::config::is_secure_url;
pub use garde::rules::url::Url;

/// Validate a string matches a URL.
pub fn url<T: Url, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    garde::rules::url::apply(value, ()).map_err(map_err)
}

/// Validate a string matches a URL and starts with https://.
pub fn url_secure<T: AsRef<str>, D, C>(
    value: T,
    data: &D,
    context: &C,
) -> Result<(), ValidateError> {
    let value = value.as_ref();

    url(&value, data, context)?;

    if !is_secure_url(value) {
        return Err(ValidateError::new("only secure URLs are allowed"));
    }

    Ok(())
}
