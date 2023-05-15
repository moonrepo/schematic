use crate::validator::ValidateError;
use garde::rules as r;

pub type Validator<T> = Box<dyn FnOnce(&T) -> Result<(), ValidateError>>;

fn map_err(error: garde::Error) -> ValidateError {
    ValidateError::new(error.to_string())
}

pub use r::alphanumeric::Alphanumeric;

pub fn alphanumeric<T: Alphanumeric>(value: &T) -> Result<(), ValidateError> {
    r::alphanumeric::apply(value, ()).map_err(map_err)
}

pub use r::ascii::Ascii;

pub fn ascii<T: Ascii>(value: &T) -> Result<(), ValidateError> {
    r::ascii::apply(value, ()).map_err(map_err)
}

pub use r::contains::Contains;

pub fn contains<T: Contains>(pattern: &str) -> Validator<T> {
    let pattern = pattern.to_owned();

    Box::new(move |value| r::contains::apply(value, (&pattern,)).map_err(map_err))
}

#[cfg(feature = "valid_email")]
pub use r::email::Email;

#[cfg(feature = "valid_email")]
pub fn email<T: Email>(value: &T) -> Result<(), ValidateError> {
    r::email::apply(value, ()).map_err(map_err)
}

pub use r::ip::{Ip, IpKind};

pub fn ip<T: Ip>(value: &T) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::Any,)).map_err(map_err)
}

pub fn ip_v4<T: Ip>(value: &T) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::V4,)).map_err(map_err)
}

pub fn ip_v6<T: Ip>(value: &T) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::V6,)).map_err(map_err)
}

pub use r::pattern::Pattern;

pub fn regex<T: Pattern>(pattern: &str) -> Validator<T> {
    let pattern = r::pattern::Regex::new(pattern).unwrap();

    Box::new(move |value| r::pattern::apply(value, (&pattern,)).map_err(map_err))
}

pub use r::length::{HasLength, Length};

pub fn min_length<T: Length>(min: usize) -> Validator<T> {
    Box::new(move |value| r::length::apply(value, (min, usize::MAX)).map_err(map_err))
}

pub fn max_length<T: Length>(max: usize) -> Validator<T> {
    Box::new(move |value| r::length::apply(value, (usize::MIN, max)).map_err(map_err))
}

pub fn in_length<T: Length>(min: usize, max: usize) -> Validator<T> {
    Box::new(move |value| r::length::apply(value, (min, max)).map_err(map_err))
}

#[cfg(feature = "valid_url")]
pub use r::url::Url;

#[cfg(feature = "valid_url")]
pub fn url<T: Url>(value: &T) -> Result<(), ValidateError> {
    r::url::apply(value, ()).map_err(map_err)
}

pub use r::range::Bounds;

pub fn in_range<T: Bounds + 'static>(min: T, max: T) -> Validator<T> {
    Box::new(move |value| r::range::apply(value, (&min, &max)).map_err(map_err))
}
