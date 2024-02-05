#[cfg(feature = "valid_email")]
mod email;
mod extends;
mod ip;
mod length;
mod number;
mod string;
#[cfg(feature = "valid_url")]
mod url;

pub use crate::config::ValidateError;
#[cfg(feature = "valid_email")]
pub use email::*;
pub use extends::*;
pub use ip::*;
pub use length::*;
pub use number::*;
pub use string::*;
#[cfg(feature = "valid_url")]
pub use url::*;

/// A validator function that receives a setting value to validate, the parent
/// configuration the setting belongs to, the current context, and can return
/// a [`ValidateError`] on failure.
pub type Validator<Val, Data, Ctx> =
    Box<dyn FnOnce(&Val, &Data, &Ctx, bool) -> Result<(), ValidateError>>;

pub(crate) fn map_err(error: garde::Error) -> ValidateError {
    ValidateError::new(error.to_string())
}
