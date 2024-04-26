#![allow(deprecated, unused_imports, unused_macros)]

use crate::*;

macro_rules! impl_unknown {
    ($type:ty) => {
        impl Schematic for $type {}
    };
}

macro_rules! impl_set {
    ($type:ty) => {
        impl<T: Schematic, S> Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.array(ArrayType::new(schema.infer::<T>()));
                schema.build()
            }
        }
    };
}

macro_rules! impl_map {
    ($type:ty) => {
        impl<K: Schematic, V: Schematic, S> Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()));
                schema.build()
            }
        }
    };
}

macro_rules! impl_string {
    ($type:ty) => {
        impl Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string(StringType::default());
                schema.build()
            }
        }
    };
}

macro_rules! impl_string_format {
    ($type:ty, $format:expr) => {
        impl Schematic for $type {
            fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string(StringType {
                    format: Some($format.into()),
                    ..StringType::default()
                });
                schema.build()
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
                fn generate_schema(mut schema: SchemaBuilder) -> Schema {
                    schema.string(StringType {
                        format: Some($format.into()),
                        ..StringType::default()
                    });
                    schema.build()
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

#[cfg(feature = "indexmap")]
mod indexmap_feature {
    use super::*;

    impl_map!(indexmap::IndexMap<K, V, S>);
    impl_set!(indexmap::IndexSet<T, S>);
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
        fn generate_schema(mut schema: SchemaBuilder) -> Schema {
            schema.integer(IntegerType::new_kind(IntegerKind::I64));
            schema.build()
        }
    }

    impl<K: Schematic, V: Schematic> Schematic for serde_json::Map<K, V> {
        fn generate_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()));
            schema.build()
        }
    }
}

#[cfg(feature = "serde_toml")]
mod serde_toml_feature {
    use super::*;

    impl_unknown!(toml::Value);

    impl<K: Schematic, V: Schematic> Schematic for toml::map::Map<K, V> {
        fn generate_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()));
            schema.build()
        }
    }
}

#[cfg(feature = "serde_yaml")]
mod serde_yaml_feature {
    use super::*;

    impl_unknown!(serde_yaml::Value);

    // This isn't accurate since we can't access the `N` enum
    impl Schematic for serde_yaml::Number {
        fn generate_schema(mut schema: SchemaBuilder) -> Schema {
            schema.integer(IntegerType::new_kind(IntegerKind::I64));
            schema.build()
        }
    }

    impl Schematic for serde_yaml::Mapping {
        fn generate_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(
                schema.infer::<serde_yaml::Value>(),
                schema.infer::<serde_yaml::Value>(),
            ));
            schema.build()
        }
    }
}

#[cfg(feature = "url")]
mod url_feature {
    use super::*;

    impl_string_format!(url::Url, "uri");
}
