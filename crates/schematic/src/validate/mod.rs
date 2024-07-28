#[cfg(feature = "validate_email")]
mod email;
#[cfg(feature = "extends")]
mod extends;
mod ip;
mod length;
mod number;
mod string;
#[cfg(feature = "validate_url")]
mod url;

pub use crate::config::{ValidateError, ValidateResult, Validator};
#[cfg(feature = "validate_email")]
pub use email::*;
#[cfg(feature = "extends")]
pub use extends::*;
pub use ip::*;
pub use length::*;
pub use number::*;
pub use string::*;
#[cfg(feature = "validate_url")]
pub use url::*;

pub(crate) fn map_err(error: garde::Error) -> ValidateError {
    ValidateError::new(error.to_string())
}
