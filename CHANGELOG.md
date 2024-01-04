# Changelog

## 0.13.2

#### 🐞 Fixes

- Fixed a build failure when all features are disabled.

## 0.13.1

#### 🐞 Fixes

- Fixed an issue where environment variables for `SchemaField` weren't being populated from
  `env_prefix`.

## 0.13.0

#### 💥 Breaking

- Refactored schema APIs for better usability.
  - Updated `TypeScriptOptions.exclude_references` and `external_types` to a `Vec` instead of
    `HashSet`.
  - Updated `EnumType.variants` to `Vec<LiteralValue>` instead of `Vec<LiteralType>`.
  - Updated `ObjectType.required` and `StructType.required` to be wrapped in `Option`.
  - Updated `SchemaField.deprecated` to `Option<String>` instead of `bool`.
  - Updated `SchemaField.name` to `String` instead of `Option<String>`.

#### 🚀 Updates

- Added official documentation: https://moonrepo.github.io/schematic
- Added a new file template generator.
- Added constructor methods for schema types.
- Added `SchemaType::enumerable` method.
- Added `SchemaField.env_var` field.
- Added `EnumType.default_index` and `UnionType.default_index` fields.
- Updated `typescript` comment output to include `@deprecated` and `@envvar`.
- Reduced the amount of code that macros generate for the `Schematic` implementation.

#### ⚙️ Internal

- Updated to Rust v1.75.
- Updated dependencies.

## 0.12.12

#### 🚀 Updates

- Updated help text to also apply for parser errors.

## 0.12.11

#### 🚀 Updates

- Added `ConfigLoader.set_help` to customize help text for validation errors.

## 0.12.10

#### 🚀 Updates

- Added serde `skip_serializing` and `skip_deserializing` support.

## 0.12.9

#### 🚀 Updates

- Added serde `flatten` support.
- Added `type_serde_json`, `type_serde_toml`, and `type_serde_yaml` features, that implements
  schematic types for serde values.

#### 🐞 Fixes

- Updated json schema unknown/any type to be a union of all types, instead of null.

#### ⚙️ Internal

- Updated dependencies.

## 0.12.8

#### ⚙️ Internal

- Updated dependencies.

## 0.12.7

#### 🐞 Fixes

- Fixed comments not rendering for enums/structs when generating TypeScript declarations.

#### ⚙️ Internal

- Updated `garde` (validation) to v0.16.
- Updated dependencies.

## 0.12.6

#### 🐞 Fixes

- Fixed `rename` on containers not being respected on generated output.

## 0.12.5

#### 🐞 Fixes

- Fixed "lowercase" and "UPPERCASE" formats incorrectly applying casing.

## 0.12.4

#### 🐞 Fixes

- Fixed a missing module error based on feature changes.

## 0.12.3

#### 🚀 Updates

- Updated enums with all unit variants to use the `Enum` schema type, instead of the `Union` schema
  type. Enums that mix and match unit with other variants will to continue to use `Union`, and will
  respect serde tagging.

#### ⚙️ Internal

- Reworked dependencies and features so that some dependencies only enable when a feature is
  enabled.

## 0.12.2

#### 🚀 Updates

- Added an `exclude` attribute for `#[setting]` and `#[schema]` that excludes the field from the
  generated schema.
  - For `Schematic`, excludes from the schema.
  - For `Config`, excludes from the schema, but is still required for the partial config.

## 0.12.1

#### 🚀 Updates

- Added `type_rust_decimal` feature, that implements schematic types for the `rust_decimal` crate.
- Added a simple caching layer for caching URL requests.
  - Added `Cacher` trait.
  - Added `Loader::set_cacher()` method.

## 0.12.0

#### 💥 Breaking

- Removed `json` as a default feature. You must now enable the file formats you want.

#### 🚀 Updates

- Added a `Schematic` derive macro that only implements the `schematic::Schematic` trait.
- Added a `config` feature that enables configuration functionality. Can use
  `default-features = false` to only use schema functionality.

#### ⚙️ Internal

- Updated Rust to v1.73.

## 0.11.8

#### 🚀 Updates

- Added support for `f32` and `f64` types.
- Added `type_version_spec` feature, that implements schematic types for the `version_spec` crate.

#### ⚙️ Internal

- Removed `Eq` from partial types so that complex types can be used.

## 0.11.7

#### 🚀 Updates

- Moved `reqwest` usage behind a feature named `url`. This is enabled by default.

#### 🐞 Fixes

- Fixed an error where Rust would fail to compile if no features are enabled.

## 0.11.6

#### ⚙️ Internal

- Updated Rust to v1.72.
- Updated `garde` (validation) to v0.15.
- Updated `reqwest` to use `rustls-tls-native-roots` instead of `rustls-tls`.
- Updated dependencies.

## 0.11.5

#### 🚀 Updates

- Added `type_warpgate` feature, that implements schematic types for the `warpgate` crate.

## 0.11.4

#### 🐞 Fixes

- Fixes a bad release.

## 0.11.3

#### 🚀 Updates

- Added `type_semver` feature, that implements schematic types for the `semver` crate.
- Added basic `#[config]` support for `ConfigEnum`.
  - Supports `rename` and `rename_all` (matches serde).
  - Added `before_parse` which transforms the string value before parsing (`From`, `FromStr`, etc).
    Supports "lowercase" and "UPPERCASE".

#### 🐞 Fixes

- Fixed serde `rename` not working on `ConfigEnum`.

## 0.11.2

#### 🚀 Updates

- Added `type_relative_path` feature, that implements schematic types for the `relative-path` crate.
- Added `type_url` feature, that implements schematic types for the `url` crate.

## 0.11.1

#### ⚙️ Internal

- Updated Rust to v1.71.
- Updated `garde` (validation) to v0.14.
- Updated dependencies.

## 0.11.0

#### 🚀 Updates

- Added support for` #[derive(Config)]` on enums with unit/tuple variants (struct variants not
  supported).
  - This allows for nested partials and configs to be properly handled.
  - Derived enums also automatically generate accurate schemas.
- Added support for `#[config(serde(...))]` attribute (doesn't support everything, mainly enum
  tagging).
- Added support for `#[setting(validate)]` on nested fields (was previously an error).
- Updated primitive schema types to include the default value.
  - Renderers will now include the default when applicable.

#### 🐞 Fixes

- Fixed a handful of issues with schema generation and partial detection.
- Fixed an issue where multiline comments weren't parsed correctly.
- Fixed an issue with string formatting that would incorrectly handle digit characters.
- Fixed a missing title/description for union types.

## 0.10.1

#### 🐞 Fixes

- Added back `Eq` and `PartialEq` to partial configs.

## 0.10.0

#### 💥 Breaking

- Renamed `SettingPath` to `Path`.
- Renamed `Segment` to `PathSegment`.
- Updated `PartialConfig.default_values` and `env_values` to return `Result<Option<Self>>` instead
  of `Result<Self>`.

#### 🚀 Updates

- Added `Deserialize` to `Format`, `Layer`, and `Source`.
- Improved file system and HTTP error handling.
- Improved and cleaned up tracing logs.
- Generator
  - Updated JSON schema arrays to use `contains` when applicable.
- Schema
  - Added support for `chrono` types (behind the `type_chrono` feature).
  - Added support for `regex` types (behind the `type_regex` feature).

#### 🐞 Fixes

- Generator
  - Updated literal types in JSON schemas to use `const`.

## 0.9.4

#### 🚀 Updates

- Generator
  - Added description to struct types.
  - Updated structs to render `additionalProperties: false` for JSON schemas.

#### 🐞 Fixes

- Generator
  - Fixed string/number/float enum values not rendering for TypeScript types.
- Schema
  - Changed nullable schemas from "one of" to "any of" union types.
  - Fixed deeply nested partial values not being marked nullable.

## 0.9.3

#### 🚀 Updates

- Generator
  - Updated struct fields to be sorted alphabetically.
  - Added `disable_references`, `exclude_references`, `external_types`, and `indent_char` options to
    TypeScript.

#### 🐞 Fixes

- Generator
  - Fixed an issue where nested enums/unions would sometimes not use references.
  - Fixed an issue with TypeScript arrays and unions. Will now wrap in parens.

## 0.9.2

#### 🐞 Fixes

- Fixed TypeScript enum/union rendering discrepancies.

## 0.9.1

#### 🐞 Fixes

- Allow the schema name to be customized after creation.
- Fixed `HashMap` and `HashSet` schematic implementations.

## 0.9.0

#### 💥 Breaking

- Removed `T::META.fields` (use `generate_schema()` instead).
- Moved the TypeScript renderer to `schematic::renderers::typescript::TypeScriptRenderer`.
  - Removed `schematic::typescript::TypeScriptGenerator`.

#### 🚀 Updates

- Added a new schema layer that defines the structure of built-in Rust types and schematic
  configuration types.
  - Implements the new `Schematic` trait.
  - Types provided by the new `schematic_types` crate.
  - Hidden behind the `schema` feature flag (very experimental).
- Added `schematic::schema::SchemaGenerator` for generating outputs from schemas.
  - Uses renderers for generating the appropriate output.
  - Moves TypeScript to a renderer.
- Added JSON schema generation.
  - Behind a new `json_schema` feature.

## 0.8.1

#### 🚀 Updates

- Added `TypeScriptGenerator::add_custom` for mapping custom types.

## 0.8.0

#### 💥 Breaking

- Renamed `ConfigMeta` trait to `Meta`.

#### 🚀 Updates

- Added `ConfigEnum` trait that the `ConfigEnum` derive macro implements.
  - Trait contains `META`.
  - Trait provides a `variants()` method.
- Added `fields` to `Meta` trait.
  - Updated `Config` trait to implement fields.
- Added TypeScript type generation (experimental, will probably change).
  - Behind a new `typescript` feature.
  - Code can be generated with the `schematic::typescript::TypeScriptGenerator`.
- Updated partials to inherit `#[allow]`, `#[warn]`, and `#[deprecated]` attributes.

#### ⚙️ Internal

- Refactored `derive_enum` maro (should be backwards compatible).
