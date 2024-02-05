use super::{map_err, Validator};
pub use garde::rules::length::{simple::Simple, HasSimpleLength};

/// Validate a value is at least the provided length.
pub fn min_length<T: Simple + HasSimpleLength, D, C>(min: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| T::validate_length(value, min, usize::MAX).map_err(map_err))
}

/// Validate a value is at most the provided length.
pub fn max_length<T: Simple + HasSimpleLength, D, C>(max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| T::validate_length(value, usize::MIN, max).map_err(map_err))
}

/// Validate a value is within the provided length.
pub fn in_length<T: Simple + HasSimpleLength, D, C>(min: usize, max: usize) -> Validator<T, D, C> {
    Box::new(move |value, _, _| T::validate_length(value, min, max).map_err(map_err))
}
