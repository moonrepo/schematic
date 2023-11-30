#![allow(deprecated, unused_imports, unused_macros)]

use crate::{IntegerKind, SchemaType, Schematic, StringType};

macro_rules! impl_unknown {
    ($type:ty) => {
        impl Schematic for $type {
            fn generate_schema() -> SchemaType {
                SchemaType::Unknown
            }
        }
    };
}

macro_rules! impl_string {
    ($type:ty) => {
        impl Schematic for $type {
            fn generate_schema() -> SchemaType {
                SchemaType::string()
            }
        }
    };
}

macro_rules! impl_string_format {
    ($type:ty, $format:expr) => {
        impl Schematic for $type {
            fn generate_schema() -> SchemaType {
                SchemaType::String(StringType {
                    format: Some($format.into()),
                    ..StringType::default()
                })
            }
        }
    };
}

#[cfg(feature = "chrono")]
mod chrono_feature {
    use super::*;

    macro_rules! impl_with_tz {
        ($type:path, $format:expr) => {
            impl<Tz: chrono::TimeZone> Schematic for $type {
                fn generate_schema() -> SchemaType {
                    SchemaType::String(StringType {
                        format: Some($format.into()),
                        ..StringType::default()
                    })
                }
            }
        };
    }

    impl_with_tz!(chrono::Date<Tz>, "date");
    impl_with_tz!(chrono::DateTime<Tz>, "date-time");
    impl_string_format!(chrono::Duration, "duration");
    impl_string_format!(chrono::Days, "duration");
    impl_string_format!(chrono::Months, "duration");
    impl_string_format!(chrono::IsoWeek, "date");
    impl_string_format!(chrono::NaiveWeek, "date");
    impl_string_format!(chrono::NaiveDate, "date");
    impl_string_format!(chrono::NaiveDateTime, "date-time");
    impl_string_format!(chrono::NaiveTime, "time");
}

#[cfg(feature = "regex")]
mod regex_feature {
    use super::*;

    impl_string_format!(regex::Regex, "regex");
}

#[cfg(feature = "relative_path")]
mod relative_path_feature {
    use super::*;

    impl_string_format!(&relative_path::RelativePath, "path");
    impl_string_format!(relative_path::RelativePath, "path");
    impl_string_format!(relative_path::RelativePathBuf, "path");
}

#[cfg(feature = "rust_decimal")]
mod rust_decimal_feature {
    use super::*;

    impl_string_format!(rust_decimal::Decimal, "decimal");
}

#[cfg(feature = "semver")]
mod semver_feature {
    use super::*;

    impl_string!(semver::Version);
    impl_string!(semver::VersionReq);
}

#[cfg(feature = "serde_json")]
mod serde_json_feature {
    use super::*;

    impl_unknown!(serde_json::Value);

    // This isn't accurate since we can't access the `N` enum
    impl Schematic for serde_json::Number {
        fn generate_schema() -> SchemaType {
            SchemaType::integer(IntegerKind::I64)
        }
    }

    impl<K: Schematic, V: Schematic> Schematic for serde_json::Map<K, V> {
        fn generate_schema() -> SchemaType {
            SchemaType::object(K::generate_schema(), V::generate_schema())
        }
    }
}

#[cfg(feature = "serde_yaml")]
mod serde_yaml_feature {
    use super::*;

    impl_unknown!(serde_yaml::Value);

    // This isn't accurate since we can't access the `N` enum
    impl Schematic for serde_yaml::Number {
        fn generate_schema() -> SchemaType {
            SchemaType::integer(IntegerKind::I64)
        }
    }

    impl Schematic for serde_yaml::Mapping {
        fn generate_schema() -> SchemaType {
            SchemaType::object(
                serde_yaml::Value::generate_schema(),
                serde_yaml::Value::generate_schema(),
            )
        }
    }
}

#[cfg(feature = "url")]
mod url_feature {
    use super::*;

    impl_string_format!(url::Url, "uri");
}

#[cfg(feature = "version_spec")]
mod version_spec_feature {
    use super::*;

    impl_string!(version_spec::UnresolvedVersionSpec);
    impl_string!(version_spec::VersionSpec);
}

#[cfg(feature = "warpgate")]
mod warpgate_feature {
    use super::*;

    impl_string!(warpgate::Id);
    impl_string!(warpgate::PluginLocator);
    impl_string_format!(warpgate::VirtualPath, "path");
}
