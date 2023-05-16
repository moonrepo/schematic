use crate::source::{is_file_like, is_url_like};
use crate::validator::{Segment, SettingPath, ValidateError};
use crate::ExtendsFrom;
use garde::rules as r;

pub type Validator<Val, Data, Ctx> =
    Box<dyn FnOnce(&Val, &Data, &Ctx) -> Result<(), ValidateError>>;

fn map_err(error: garde::Error) -> ValidateError {
    ValidateError::new(error.to_string())
}

pub use r::alphanumeric::Alphanumeric;

/// Validate a string is only composed of alpha-numeric characters.
pub fn alphanumeric<T: Alphanumeric, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::alphanumeric::apply(value, ()).map_err(map_err)
}

pub use r::ascii::Ascii;

/// Validate a string is only composed of ASCII characters.
pub fn ascii<T: Ascii, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::ascii::apply(value, ()).map_err(map_err)
}

pub use r::contains::Contains;

/// Validate a string contains the provided pattern.
pub fn contains<T: Contains, D, C>(pattern: &str) -> Validator<T, D, C> {
    let pattern = pattern.to_owned();

    Box::new(move |value, _, _| r::contains::apply(value, (&pattern,)).map_err(map_err))
}

#[cfg(feature = "valid_email")]
pub use r::email::Email;

#[cfg(feature = "valid_email")]
/// Validate a string matches an email address.
pub fn email<T: Email, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::email::apply(value, ()).map_err(map_err)
}

pub use r::ip::{Ip, IpKind};

/// Validate a string is either an IP v4 or v6 address.
pub fn ip<T: Ip, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::Any,)).map_err(map_err)
}

/// Validate a string is either an IP v4 address.
pub fn ip_v4<T: Ip, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::V4,)).map_err(map_err)
}

/// Validate a string is either an IP v6 address.
pub fn ip_v6<T: Ip, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::ip::apply(value, (IpKind::V6,)).map_err(map_err)
}

pub use r::pattern::Pattern;

/// Validate a string matches the provided regex pattern.
pub fn regex<T: Pattern, D, C>(pattern: &str) -> Validator<T, D, C> {
    let pattern = r::pattern::Regex::new(pattern).unwrap();

    Box::new(move |value, _, _| r::pattern::apply(value, (&pattern,)).map_err(map_err))
}

pub use r::length::{HasLength, Length};

/// Validate a value is at least the provided length.
pub fn min_length<T: Length, D, C>(min: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| r::length::apply(value, (min, usize::MAX)).map_err(map_err))
}

/// Validate a value is at most the provided length.
pub fn max_length<T: Length, D, C>(max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| r::length::apply(value, (usize::MIN, max)).map_err(map_err))
}

/// Validate a value is within the provided length.
pub fn in_length<T: Length, D, C>(min: usize, max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| r::length::apply(value, (min, max)).map_err(map_err))
}

/// Validate the value is not empty.
pub fn not_empty<T: HasLength, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    if value.length() == 0 {
        return Err(ValidateError::new("must not be empty"));
    }

    Ok(())
}

#[cfg(feature = "valid_url")]
pub use r::url::Url;

#[cfg(feature = "valid_url")]
/// Validate a string matches a URL.
pub fn url<T: Url, D, C>(value: &T, _: &D, _: &C) -> Result<(), ValidateError> {
    r::url::apply(value, ()).map_err(map_err)
}

#[cfg(feature = "valid_url")]
pub fn url_secure<T: AsRef<str>, D, C>(
    value: T,
    data: &D,
    context: &C,
) -> Result<(), ValidateError> {
    url(&value, data, context)?;

    let value = value.as_ref();

    if !value.starts_with("https://") && !value.contains("127.0.0.1") {
        return Err(ValidateError::new("only secure URLs are allowed"));
    }

    Ok(())
}

pub use r::range::Bounds;

/// Validate a numeric value is between the provided bounds (non-inclusive).
pub fn in_range<T: Bounds + 'static, D, C>(min: T, max: T) -> Validator<T, D, C> {
    Box::new(move |value, _, _| r::range::apply(value, (&min, &max)).map_err(map_err))
}

/// Validate an extends value is either a file path or secure URL.
#[allow(unused_variables)]
pub fn extends_string<D, C>(value: &str, data: &D, context: &C) -> Result<(), ValidateError> {
    #[allow(unreachable_code)]
    if is_url_like(value) {
        #[cfg(feature = "valid_url")]
        {
            return url_secure(value, data, context);
        }

        return Ok(());
    } else if is_file_like(value) {
        return Ok(());
    }

    Err(ValidateError::new(
        "only file paths and URLs can be extended",
    ))
}

/// Validate a list of extends values are either a file path or secure URL.
pub fn extends_list<D, C>(values: &[String], data: &D, context: &C) -> Result<(), ValidateError> {
    for (i, value) in values.iter().enumerate() {
        if let Err(mut error) = extends_string(value, data, context) {
            error.path = Some(SettingPath::new(vec![Segment::Index(i)]));

            return Err(error);
        }
    }

    Ok(())
}

/// Validate an extends value(s) is either a file path or secure URL.
pub fn extends_from<D, C>(value: &ExtendsFrom, data: &D, context: &C) -> Result<(), ValidateError> {
    match value {
        ExtendsFrom::String(string) => extends_string(string, data, context)?,
        ExtendsFrom::List(list) => extends_list(list, data, context)?,
    };

    Ok(())
}
