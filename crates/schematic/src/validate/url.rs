use super::{ValidateError, ValidateResult, map_err};
use crate::helpers::is_secure_url;
pub use garde::rules::url::Url;

/// Validate a string matches a URL.
pub fn url<T: Url, D, C>(value: &T, _data: &D, _context: &C, _finalize: bool) -> ValidateResult {
    garde::rules::url::apply(value, ()).map_err(map_err)
}

/// Validate a string matches a URL and starts with https://.
pub fn url_secure<T: AsRef<str>, D, C>(
    value: T,
    data: &D,
    context: &C,
    finalize: bool,
) -> ValidateResult {
    let value = value.as_ref();

    url(&value, data, context, finalize)?;

    if !is_secure_url(value) {
        return Err(ValidateError::new("only secure URLs are allowed"));
    }

    Ok(())
}
