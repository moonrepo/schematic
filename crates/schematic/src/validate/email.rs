use super::{map_err, ValidateError};
pub use garde::rules::email::Email;

/// Validate a string matches an email address.
pub fn email<T: Email, D, C>(
    value: &T,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    garde::rules::email::apply(value, ()).map_err(map_err)
}
