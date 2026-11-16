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
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.array(ArrayType {
                    items_type: Box::new(schema.infer::<T>()),
                    unique: Some(true),
                    ..ArrayType::default()
                })
            }
        }
    };
}

macro_rules! impl_map {
    ($type:ty) => {
        impl<K: Schematic, V: Schematic, S> Schematic for $type {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()))
            }
        }
    };
}

macro_rules! impl_string {
    ($type:ty) => {
        impl Schematic for $type {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string_default()
            }
        }
    };
}

macro_rules! impl_string_format {
    ($type:ty, $format:expr_2021) => {
        impl Schematic for $type {
            fn build_schema(mut schema: SchemaBuilder) -> Schema {
                schema.string(StringType {
                    format: Some($format.into()),
                    ..StringType::default()
                })
            }
        }
    };
}

#[cfg(feature = "arrayvec")]
mod arrayvec_feature {
    use super::*;

    // Deserializing more items than it can hold fails
    impl<T: Schematic, const CAP: usize> Schematic for arrayvec::ArrayVec<T, CAP> {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.array(ArrayType {
                items_type: Box::new(schema.infer::<T>()),
                max_length: Some(CAP),
                ..ArrayType::default()
            })
        }
    }

    // Its capacity is in bytes, while a schema counts characters, of which
    // there are never more, so the bound is loose for anything but ASCII
    impl<const CAP: usize> Schematic for arrayvec::ArrayString<CAP> {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.string(StringType {
                max_length: Some(CAP),
                ..StringType::default()
            })
        }
    }
}

#[cfg(feature = "chrono")]
mod chrono_feature {
    use super::*;

    macro_rules! impl_with_tz {
        ($type:path, $format:expr_2021) => {
            impl<Tz: chrono::TimeZone> Schematic for $type {
                fn build_schema(mut schema: SchemaBuilder) -> Schema {
                    schema.string(StringType {
                        format: Some($format.into()),
                        ..StringType::default()
                    })
                }
            }
        };
    }

    impl_with_tz!(chrono::Date<Tz>, "date");
    impl_with_tz!(chrono::DateTime<Tz>, "date-time");
    impl_string_format!(chrono::Days, "duration");
    impl_string_format!(chrono::Months, "duration");
    impl_string_format!(chrono::IsoWeek, "date");
    impl_string_format!(chrono::NaiveWeek, "date");
    impl_string_format!(chrono::NaiveDate, "date");

    // These have no offset, which the `date-time` and `time` formats
    // require, so they're plain strings, like `2024-01-02T03:04:05`
    impl_string!(chrono::NaiveDateTime);
    impl_string!(chrono::NaiveTime);

    // Serialized as a tuple of seconds and nanoseconds, not a string
    impl Schematic for chrono::Duration {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.tuple(TupleType::new([
                schema.infer::<i64>(),
                schema.infer::<i32>(),
            ]))
        }
    }
}

#[cfg(feature = "compact_str")]
mod compact_str_feature {
    use super::*;

    impl_string!(compact_str::CompactString);
}

#[cfg(feature = "hashbrown")]
mod hashbrown_feature {
    use super::*;

    impl_map!(hashbrown::HashMap<K, V, S>);
    impl_set!(hashbrown::HashSet<T, S>);
}

#[cfg(feature = "indexmap")]
mod indexmap_feature {
    use super::*;

    impl_map!(indexmap::IndexMap<K, V, S>);
    impl_set!(indexmap::IndexSet<T, S>);
}

#[cfg(feature = "jiff")]
mod jiff_feature {
    use super::*;

    impl_string_format!(jiff::Timestamp, "date-time");
    impl_string_format!(jiff::civil::Date, "date");

    // These have no offset, which the `date-time` and `time` formats
    // require, so they're plain strings, like `2024-01-02T03:04:05`
    impl_string!(jiff::civil::DateTime);
    impl_string!(jiff::civil::Time);

    // A week, like `2024-W01-2`, which isn't a `date`
    impl_string!(jiff::civil::ISOWeekDate);

    // The offset is followed by the time zone in brackets, like
    // `2024-01-02T03:04:05-05:00[America/New_York]`, which isn't a `date-time`
    impl_string!(jiff::ZonedDateTime);

    // Written in ISO 8601, like `PT1H30M`, but also read in jiff's friendly
    // format, like `1h 30m`, which the `duration` format would reject
    impl_string!(jiff::SignedDuration);
    impl_string!(jiff::Span);
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

    // Serialized as a string, while a number is also accepted
    impl Schematic for rust_decimal::Decimal {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.union(UnionType::new_any([
                Schema::string(StringType {
                    format: Some("decimal".into()),
                    ..StringType::default()
                }),
                Schema::float(FloatType::new_kind(FloatKind::F64)),
            ]))
        }
    }
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

    // Either an integer or a float, which a float type describes
    impl Schematic for serde_json::Number {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.float(FloatType::new_kind(FloatKind::F64))
        }
    }

    impl<K: Schematic, V: Schematic> Schematic for serde_json::Map<K, V> {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()))
        }
    }
}

#[cfg(feature = "serde_ron")]
mod serde_ron_feature {
    use super::*;

    impl_unknown!(ron::Value);
}

#[cfg(feature = "serde_rpkl")]
mod serde_rpkl_feature {
    use super::*;

    impl_unknown!(rpkl::Value);
}

#[cfg(feature = "serde_toml")]
mod serde_toml_feature {
    use super::*;

    impl_unknown!(toml::Value);

    impl<K: Schematic, V: Schematic> Schematic for toml::map::Map<K, V> {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(schema.infer::<K>(), schema.infer::<V>()))
        }
    }
}

#[cfg(feature = "serde_yaml")]
mod serde_yaml_feature {
    use super::*;

    impl_unknown!(serde_yaml::Value);

    // Either an integer or a float, which a float type describes
    impl Schematic for serde_yaml::Number {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.float(FloatType::new_kind(FloatKind::F64))
        }
    }

    impl Schematic for serde_yaml::Mapping {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(
                schema.infer::<serde_yaml::Value>(),
                schema.infer::<serde_yaml::Value>(),
            ))
        }
    }
}

#[cfg(feature = "serde_yaml_norway")]
mod serde_yaml_normay_feature {
    use super::*;

    impl_unknown!(serde_norway::Value);

    // Either an integer or a float, which a float type describes
    impl Schematic for serde_norway::Number {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.float(FloatType::new_kind(FloatKind::F64))
        }
    }

    impl Schematic for serde_norway::Mapping {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.object(ObjectType::new(
                schema.infer::<serde_norway::Value>(),
                schema.infer::<serde_norway::Value>(),
            ))
        }
    }
}

#[cfg(feature = "smallvec")]
mod smallvec_feature {
    use super::*;

    // Its capacity is only what's held inline, as it spills onto the heap
    impl<A: smallvec::Array> Schematic for smallvec::SmallVec<A>
    where
        A::Item: Schematic,
    {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.array(ArrayType::new(schema.infer::<A::Item>()))
        }
    }
}

#[cfg(feature = "smol_str")]
mod smol_str_feature {
    use super::*;

    impl_string!(smol_str::SmolStr);
}

#[cfg(feature = "url")]
mod url_feature {
    use super::*;

    impl_string_format!(url::Url, "uri");
}

#[cfg(feature = "uuid")]
mod uuid_feature {
    use super::*;

    impl_string_format!(uuid::Uuid, "uuid");
}
