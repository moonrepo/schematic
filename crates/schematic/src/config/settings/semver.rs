use crate::schema::{Schema, SchemaBuilder, Schematic};
use semver::{Error, Version};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct VersionSetting(pub Version);

impl VersionSetting {
    pub fn new(value: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Self(Version::parse(value.as_ref())?))
    }
}

impl Default for VersionSetting {
    fn default() -> Self {
        Self(Version::new(0, 0, 0))
    }
}

impl Deref for VersionSetting {
    type Target = Version;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for VersionSetting {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for VersionSetting {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for VersionSetting {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for VersionSetting {
    fn into(self) -> String {
        self.to_string()
    }
}

impl PartialEq<VersionSetting> for VersionSetting {
    fn eq(&self, other: &VersionSetting) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for VersionSetting {}

impl Hash for VersionSetting {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Schematic for VersionSetting {
    fn build_schema(_: SchemaBuilder) -> Schema {
        SchemaBuilder::generate::<Version>()
    }
}
