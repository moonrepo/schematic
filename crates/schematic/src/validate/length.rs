use super::{Validator, map_err};
pub use garde::rules::length::{
    HasSimpleLength,
    bytes::{Bytes, HasBytes},
    chars::{Chars, HasChars},
    simple::Simple,
};

/// Validate a value is within the provided length.
pub fn in_length<T: Simple + HasSimpleLength, D, C>(min: usize, max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_length(value, min, max).map_err(map_err))
}

/// Validate a value is at least the provided length.
pub fn min_length<T: Simple + HasSimpleLength, D, C>(min: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_length(value, min, usize::MAX).map_err(map_err))
}

/// Validate a value is at most the provided length.
pub fn max_length<T: Simple + HasSimpleLength, D, C>(max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_length(value, usize::MIN, max).map_err(map_err))
}

/// Validate a value has the minimum required number of characters.
pub fn min_chars<T: Chars + HasChars, D, C>(min: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_num_chars(value, min, usize::MAX).map_err(map_err))
}

/// Validate a value has the maximum required number of characters.
pub fn max_chars<T: Chars + HasChars, D, C>(max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_num_chars(value, usize::MIN, max).map_err(map_err))
}

/// Validate a value has the minimum required number of bytes.
pub fn min_bytes<T: Bytes + HasBytes, D, C>(min: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_num_bytes(value, min, usize::MAX).map_err(map_err))
}

/// Validate a value has the maximum required number of bytes.
pub fn max_bytes<T: Bytes + HasBytes, D, C>(max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| T::validate_num_bytes(value, usize::MIN, max).map_err(map_err))
}
