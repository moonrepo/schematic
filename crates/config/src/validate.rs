use crate::validator::ValidateError;

pub type Validator<T> = Box<dyn FnOnce(&T) -> Result<(), ValidateError>>;

fn map_err(error: garde::Error) -> ValidateError {
    ValidateError::new(error.to_string())
}

pub use garde::rules::alphanumeric::Alphanumeric;

pub fn alphanumeric<T: Alphanumeric>(value: &T) -> Result<(), ValidateError> {
    garde::rules::alphanumeric::apply(value, ()).map_err(map_err)
}

pub use garde::rules::ascii::Ascii;

pub fn ascii<T: Ascii>(value: &T) -> Result<(), ValidateError> {
    garde::rules::ascii::apply(value, ()).map_err(map_err)
}

pub use garde::rules::contains::Contains;

pub fn contains<T: Contains>(pattern: &str) -> Validator<T> {
    let pattern = pattern.to_owned();

    Box::new(move |value| garde::rules::contains::apply(value, (&pattern,)).map_err(map_err))
}

#[cfg(feature = "valid_email")]
pub use garde::rules::email::Email;

#[cfg(feature = "valid_email")]
pub fn email<T: Email>(value: &T) -> Result<(), ValidateError> {
    garde::rules::email::apply(value, ()).map_err(map_err)
}

pub use garde::rules::ip::{Ip, IpKind};

pub fn ip<T: Ip>(value: &T) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::Any,)).map_err(map_err)
}

pub fn ip_v4<T: Ip>(value: &T) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::V4,)).map_err(map_err)
}

pub fn ip_v6<T: Ip>(value: &T) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::V6,)).map_err(map_err)
}
