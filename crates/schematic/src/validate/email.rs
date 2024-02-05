use super::{map_err, ValidateError};
pub use garde::rules::email::Email;

/// Validate a string matches an email address.
pub fn email<T: Email, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    garde::rules::email::apply(value, ()).map_err(map_err)
}
