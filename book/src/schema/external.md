# External types

Schematic provides schema implementations for third-party [crates](https://crates.io) through a
concept known as external types. This functionality is opt-in through Cargo features.

Each feature implements the version of the crate that Schematic depends on, so the version in your
own `Cargo.toml` must be compatible with it, or the type won't implement `Schematic`.

## arrayvec

> Requires the `type_arrayvec` Cargo feature, and arrayvec 0.7.

Implements schemas for `ArrayVec` and `ArrayString` from the
[arrayvec](https://crates.io/crates/arrayvec) crate. Each one's capacity is its maximum length,
which for an `ArrayString` is in bytes, so a schema only rejects a string with more characters than
that.

## chrono

> Requires the `type_chrono` Cargo feature.

Implements schemas for `Date`, `DateTime`, `Duration`, `Days`, `Months`, `IsoWeek`, `NaiveWeek`,
`NaiveDate`, `NaiveDateTime`, and `NaiveTime` from the [chrono](https://crates.io/crates/chrono)
crate.

## compact_str

> Requires the `type_compact_str` Cargo feature, and compact_str 0.10.

Implements a schema for `CompactString` from the [compact_str](https://crates.io/crates/compact_str)
crate.

## hashbrown

> Requires the `type_hashbrown` Cargo feature, and hashbrown 0.17.

Implements schemas for `HashMap` and `HashSet` from the
[hashbrown](https://crates.io/crates/hashbrown) crate.

## indexmap

> Requires the `type_indexmap` Cargo feature.

Implements a schema for `IndexMap` and `IndexSet` from the
[indexmap](https://crates.io/crates/indexmap) crate.

## jiff

> Requires the `type_jiff` Cargo feature, and jiff 1.

Implements schemas for `Timestamp`, `ZonedDateTime`, `Span`, and `SignedDuration`, and the civil
`Date`, `DateTime`, `Time`, and `ISOWeekDate`, from the [jiff](https://crates.io/crates/jiff) crate.

Only `Timestamp` and `Date` have a format, `date-time` and `date`. The others are strings, as their
format would reject what jiff writes or reads: a `ZonedDateTime` ends with its time zone, like
`[America/New_York]`, a civil `DateTime` and `Time` have no offset, an `ISOWeekDate` is a week, like
`2024-W01-2`, and a `Span` and `SignedDuration` are also read in jiff's friendly format, like
`1h 30m`.

## regex

> Requires the `type_regex` Cargo feature.

Implements a schema for `Regex` from the [regex](https://crates.io/crates/regex) crate.

> To use `Regex` as a [setting](../config/settings.md#third-party-types), reach for `RegexSetting`, which
> supplies the `Default` a required setting needs.

## relative-path

> Requires the `type_relative_path` Cargo feature.

Implements schemas for `RelativePath` and `RelativePathBuf` from the
[relative-path](https://crates.io/crates/relative-path) crate.

## rpkl

> Requires the `pkl` Cargo feature.

Implements a schema for `Value` from the [rpkl](https://crates.io/crates/rpkl) crate.

## rust_decimal

> Requires the `type_rust_decimal` Cargo feature.

Implements a schema for `Decimal` from the [rust_decimal](https://crates.io/crates/rust_decimal)
crate.

## semver

> Requires the `type_semver` Cargo feature.

Implements schemas for `Version` and `VersionReq` from the [semver](https://crates.io/crates/semver)
crate.

> To use `Version` as a [setting](../config/settings.md#third-party-types), reach for `VersionSetting`, which
> supplies the `Default` a required setting needs.

## ron

> Requires the `ron` Cargo feature.

Implements a schema for `Value` from the [ron](https://crates.io/crates/ron) crate.

## serde_json

> Requires the `json` Cargo feature.

Implements schemas for `Value`, `Number`, and `Map` from the
[serde_json](https://crates.io/crates/serde_json) crate.

## serde_yaml

> Requires the `serde_yaml` Cargo feature on `schematic_types` directly. The `schematic` crate does
> not enable it, as its `yaml` feature uses `serde_norway` instead.

Implements schemas for `Value`, `Number`, and `Mapping` from the
[serde_yaml](https://crates.io/crates/serde_yaml) crate.

## serde_yaml_norway

> Requires the `yaml` Cargo feature.

Implements schemas for `Value`, `Number`, and `Mapping` from the
[serde_norway](https://crates.io/crates/serde_norway) crate.

## smallvec

> Requires the `type_smallvec` Cargo feature, and smallvec 1.

Implements a schema for `SmallVec` from the [smallvec](https://crates.io/crates/smallvec) crate.

## smol_str

> Requires the `type_smol_str` Cargo feature, and smol_str 0.3.

Implements a schema for `SmolStr` from the [smol_str](https://crates.io/crates/smol_str) crate.

## toml

> Requires the `toml` Cargo feature.

Implements schemas for `Value` and `Map` from the [toml](https://crates.io/crates/toml) crate.

## url

> Requires the `type_url` Cargo feature.

Implements a schema for `Url` from the [url](https://crates.io/crates/url) crate.

## uuid

> Requires the `type_uuid` Cargo feature.

Implements a schema for `Uuid` from the [uuid](https://crates.io/crates/uuid) crate.
