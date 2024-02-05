use super::{map_err, ValidateError};
pub use garde::rules::ip::{Ip, IpKind};

/// Validate a string is either an IP v4 or v6 address.
pub fn ip<T: Ip, D, C>(
    value: &T,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::Any,)).map_err(map_err)
}

/// Validate a string is either an IP v4 address.
pub fn ip_v4<T: Ip, D, C>(
    value: &T,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::V4,)).map_err(map_err)
}

/// Validate a string is either an IP v6 address.
pub fn ip_v6<T: Ip, D, C>(
    value: &T,
    _data: &D,
    _context: &C,
    _finalize: bool,
) -> Result<(), ValidateError> {
    garde::rules::ip::apply(value, (IpKind::V6,)).map_err(map_err)
}
