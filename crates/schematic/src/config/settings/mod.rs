//! Newtypes for third-party types that cannot be used as a setting as-is.
//!
//! A setting that isn't wrapped in `Option` falls back to `Default::default()`
//! when the config provides no value and the field declares no
//! `#[setting(default)]`, so every required setting has to implement `Default`.
//! The types wrapped here don't, and being foreign, can't have it implemented
//! for them directly.
//!
//! Each wrapper picks a sensible default, derefs to the inner type so it reads
//! like the original, and (de)serializes through a string, which also frees it
//! from the upstream crate's `serde` feature.

#[cfg(feature = "type_regex")]
mod regex;
#[cfg(feature = "type_semver")]
mod semver;

#[cfg(feature = "type_regex")]
pub use regex::*;
#[cfg(feature = "type_semver")]
pub use semver::*;
