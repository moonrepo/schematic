#[cfg(feature = "schema")]
use crate::schema::{Schema, SchemaBuilder, Schematic};
use regex::{Error, Regex};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::str::FromStr;

/// A [`Regex`] that can be used as a setting.
///
/// [`Regex`] implements none of `Default`, `PartialEq`, `Serialize`, or
/// `Deserialize`, all of which a required setting needs, and cannot be given
/// them from here as it is a foreign type. This wrapper provides them, along
/// with `Eq` and `Hash` for convenience, and compares by pattern string since
/// a compiled regex has no equality of its own.
///
/// Defaults to `.`, which matches any single character.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct RegexSetting(pub Regex);

impl RegexSetting {
    /// Compile a regex from a pattern string.
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Self(Regex::new(value.as_ref())?))
    }
}

/// Defaults to `.`, which matches any single character.
impl Default for RegexSetting {
    fn default() -> Self {
        Self(Regex::new(".").unwrap())
    }
}

/// Derefs to the inner [`Regex`], so its methods can be called directly.
impl Deref for RegexSetting {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for RegexSetting {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for RegexSetting {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for RegexSetting {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for RegexSetting {
    fn into(self) -> String {
        self.to_string()
    }
}

/// Compares the pattern strings, as a compiled [`Regex`] is not comparable.
impl PartialEq<RegexSetting> for RegexSetting {
    fn eq(&self, other: &RegexSetting) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for RegexSetting {}

impl Hash for RegexSetting {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.0.as_str().as_bytes());
    }
}

#[cfg(feature = "schema")]
impl Schematic for RegexSetting {
    fn build_schema(_: SchemaBuilder) -> Schema {
        SchemaBuilder::generate::<Regex>()
    }
}
