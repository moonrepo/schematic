# External types

Schematic provides schema implementations for third-party [crates](https://crates.io) through a
concept known as external types. This functionality is opt-in through Cargo features.

## chrono

> Requires the `type_chrono` Cargo feature.

Implements schemas for `Date`, `DateTime`, `Duration`, `Days`, `Months`, `IsoWeek`, `NaiveWeek`,
`NaiveDate`, `NaiveDateTime`, and `NaiveTime` from the [chrono](https://crates.io/crates/chrono)
crate.

## indexmap

> Requires the `type_indexmap` Cargo feature.

Implements a schema for `IndexMap` and `IndexSet` from the
[indexmap](https://crates.io/crates/indexmap) crate.

## regex

> Requires the `type_regex` Cargo feature.

Implements a schema for `Regex` from the [regex](https://crates.io/crates/regex) crate.

## relative-path

> Requires the `type_relative_path` Cargo feature.

Implements schemas for `RelativePath` and `RelativePathBuf` from the
[relative-path](https://crates.io/crates/relative-path) crate.

## rpkl

> Requires the `type_serde_rpkl` Cargo feature.

Implements schemas for `Value` from the [rpkl](https://crates.io/crates/rpkl) crate.

## rust_decimal

> Requires the `type_rust_decimal` Cargo feature.

Implements a schema for `Decimal` from the [rust_decimal](https://crates.io/crates/rust_decimal)
crate.

## semver

> Requires the `type_semver` Cargo feature.

Implements schemas for `Version` and `VersionReq` from the [semver](https://crates.io/crates/semver)
crate.

## serde_json

> Requires the `type_serde_json` Cargo feature.

Implements schemas for `Value`, `Number`, and `Map` from the
[serde_json](https://crates.io/crates/serde_json) crate.

## serde_yaml

> Requires the `type_serde_yaml` Cargo feature.

Implements schemas for `Value`, `Number`, and `Mapping` from the
[serde_yaml](https://crates.io/crates/serde_yaml) crate.

## toml

> Requires the `type_serde_toml` Cargo feature.

Implements schemas for `Value` and `Map` from the [toml](https://crates.io/crates/toml) crate.

## url

> Requires the `type_url` Cargo feature.

Implements a schema for `Url` from the [url](https://crates.io/crates/url) crate.

## uuid

> Requires the `type_uuid` Cargo feature.

Implements a schema for `Uuid` from the [uuid](https://crates.io/crates/uuid) crate.
