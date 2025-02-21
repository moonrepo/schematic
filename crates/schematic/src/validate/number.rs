use super::{Validator, map_err};
pub use garde::rules::range::Bounds;
use std::fmt::Display;

/// Validate a numeric value is between the provided bounds (non-inclusive).
pub fn in_range<T: Bounds + Display + 'static, D, C>(
    min: T::Size,
    max: T::Size,
) -> Validator<T, D, C> {
    Box::new(move |value, _, _, _| {
        garde::rules::range::apply(value, (Some(min), Some(max))).map_err(map_err)
    })
}
