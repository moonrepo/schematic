#[cfg(feature = "type_regex")]
mod regex;
#[cfg(feature = "type_semver")]
mod semver;

#[cfg(feature = "type_regex")]
pub use regex::*;
#[cfg(feature = "type_semver")]
pub use semver::*;
