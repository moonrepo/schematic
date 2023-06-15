# Changelog

## Unreleased

#### 💥 Breaking

- Renamed `SettingPath` to `Path`.
- Renamed `Segment` to `PathSegment`.

#### 🚀 Updates

- Generator
  - Updated JSON schema arrays to use `contains` when applicable.

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
