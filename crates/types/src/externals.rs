#![allow(deprecated, unused_imports, unused_macros)]

use crate::{SchemaType, Schematic, StringType};

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

#[cfg(feature = "relative-path")]
mod relative_path_feature {
    use super::*;

    impl_string_format!(&relative_path::RelativePath, "path");
    impl_string_format!(relative_path::RelativePath, "path");
    impl_string_format!(relative_path::RelativePathBuf, "path");
}

#[cfg(feature = "semver")]
mod semver_feature {
    use super::*;

    impl_string!(semver::Version);
    impl_string!(semver::VersionReq);
}

#[cfg(feature = "url")]
mod url_feature {
    use super::*;

    impl_string_format!(url::Url, "uri");
}
