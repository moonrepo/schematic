# Changelog

## Unreleased

#### 🚀 Updates

- Added `type_semver` feature, that implements schematic types for the `semver` crate.

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
