# Changelog

## Next

#### 🚀 Updates

- Added support for explicit Serde deserialize and serialize renaming on containers and fields:
  `#[serde(rename(deserialize = "de_name", serialize = "ser_name"))]`.
- Updated Serde `alias` to support multiple aliases: `#[serde(alias = "alias1", alias = "alias2")]`

## 0.18.12

#### 🚀 Updates

- Added `SchemaField.aliases` property.
- Added aliases support to JSONSchema and TypeScript renderers.
  - Added an `exclude_aliases` option to disable inclusion.
- Added a `VersionSetting` type that wraps `semver::Version` and implements additional traits so
  that it can be used in configs.
  - Requires the `type_semver` feature.
  - Default implementation uses `0.0.0`.

#### ⚙️ Internal

- Updated dependencies.

## 0.18.11

#### 🚀 Updates

- Added a `RegexSetting` type that wraps `regex::Regex` and implements additional traits so that it
  can be used in configs.
  - Requires the `type_regex` feature.
  - Default implementation uses `.` (match all) pattern.

## 0.18.10

#### ⚙️ Internal

- Updated to Rust v1.88.
- Updated dependencies.

## 0.18.9

#### 🚀 Updates

- Added RON support (`.ron` files) behind the `ron` feature.
- Added `type_uuid` feature for UUID schema support.

## 0.18.8

#### 🚀 Updates

- Added `#[config(partial())]` to containers, allow derives to be set and applied to the generated
  partial configs.

## 0.18.7

#### 🐞 Fixes

- Fixed an issue where nested optional configs (`setting: Option<Config>`) would not be marked as
  some when an applicable environment variable is set.

## 0.18.6

#### 🚀 Updates

- Added `Display` to all schema types, so that TypeScript-like type strings can be generated.

#### ⚙️ Internal

- Updated dependencies.

## 0.18.5

#### ⚙️ Internal

- Updated dependencies.

## 0.18.4

#### ⚙️ Internal

- Updated dependencies.

## 0.18.3

#### 🚀 Updates

- Support parsing files with a byte-order mark (BOM).

#### ⚙️ Internal

- Updated to Rust v1.86.

## 0.18.2

#### ⚙️ Internal

- Updated dependencies.

## 0.18.1

#### 🚀 Updates

- Added `#[setting(transform)]`, which allows defined values to be transformed before being
  finalized.
- Added support for `#[setting(default)]` to `Option`al settings.

#### ⚙️ Internal

- Updated dependencies.

## 0.18.0

#### 🚀 Updates

- Updated to Rust v1.85 and 2024 edition.

#### 🐞 Fixes

- Fixed an issue where importing `Result` would break our generated macro code.

#### ⚙️ Internal

- Updated dependencies.

## 0.17.11

#### 🚀 Updates

- When using the `pkl` format, we now check for the `pkl` binary, and error if not found.

## 0.17.10

#### 🚀 Updates

- Added `yml` and `serde_yml` features, which uses `serde_yml` for parsing instead of `serde_yaml`,
  as the latter has been deprecated for sometime.

#### ⚙️ Internal

- Updated `garde` (validation) to v0.22.
- Updated dependencies.

## 0.17.9

#### 🚀 Updates

- Added `ConfigSetting.nested` to recursively include nested settings.

## 0.17.8

#### 🚀 Updates

- Added `Config::settings()`, which returns basic metadata about settings on the config.

#### 🐞 Fixes

- Fixed an issue where TypeScript imports weren't sorted and the order would randomize.

#### ⚙️ Internal

- Updated `garde` (validation) to v0.21.
- Updated dependencies.

## 0.17.7

#### 🐞 Fixes

- Fixed more template rendering issues.

#### ⚙️ Internal

- Updated dependencies.
- Updated Rust to v1.83 (for development).

## 0.17.6

#### 🐞 Fixes

- Fixed an issue where the YAML template rendered wouldn't nest fields correctly.

#### ⚙️ Internal

- Updated dependencies.
- Updated Rust to v1.82 (for development).

## 0.17.5

#### 🐞 Fixes

- Bubble up Pkl error when applicable.

## 0.17.4

#### 🚀 Updates

- Added support for `#[serde(deny_unknown_fields)]` to struct and enum containers.
- Added support for `#[serde(rename_all_fields)]` to enum containers.
- Added support for `#[serde(untagged)]` to enum variants.

#### ⚙️ Internal

- Updated dependencies.

## 0.17.3

#### 🐞 Fixes

- More serde optimizations and fixes.

## 0.17.2

#### 🚀 Updates

- Updated `Schema`, `SchemaField`, and `SchemaType` to implement `Schematic`. Right now they just
  default to "unknown" shapes, but was added so that composition doesn't error.

#### ⚙️ Internal

- Updated dependencies.

## 0.17.1

#### 🐞 Fixes

- Fixed some issues around serializing the schemas.

## 0.17.0

#### 💥 Breaking

##### Config

- Refactored the internals of how merge/validation errors work.
- Removed `Config::META` and `ConfigError::META`. Use `Schematic::schema_name()` instead.
- Removed `url` as a default Cargo feature.
- Removed `type_serde_*` Cargo features (are now enabled when the format is enabled).
- Renamed `valid_*` Cargo features to `validate_*`.
- Renamed some error enum variants.

##### Schema

- Updated `SchemaRenderer::render` to receive an owned copy of the schemas.
- Removed references from `SchemaRenderer::render`. Use `schemas` keys instead.
- Removed generics from `SchemaGenerator` and `SchemaRenderer`.

```rust
// Before
fn render(
  &mut self,
  schemas: &'gen IndexMap<String, Schema>,
  references: &'gen HashSet<String>,
) -> RenderResult;

// After
fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult;
```

#### 🚀 Updates

- Added experimental support for the [Pkl configuration language](https://pkl-lang.org/) (`.pkl`
  files).
  - There are caveats to using Pkl, please refer to the docs.
- Added a `pkl` Cargo feature to enable the Pkl format.
- Added a `env` Cargo feature for toggling environment variable functionality. Enabled by default.
- Added a `extends` Cargo feature for config extending functionality. Enabled by default.
- Added a `validate` Cargo feature for toggling validation functionality. Enabled by default.
- Added a `schema_serde` Cargo feature for allowing the `Schema` to be serialized.
- Reworked how parser and validator errors are rendered in the terminal.

#### ⚙️ Internal

- Added an MSRV requirement for v1.71.1.
- Updated Rust to v1.80 (for development).

## 0.16.6

#### 🚀 Updates

- We now track `#[deprecated]` on containers.
- TypeScript
  - When rendering a struct field that has an enum type, we'll include a `@type` doc tag for all
    variants.
  - Deprecated structs (interfaces) and enums will now contain `@deprecated` on the container.

#### ⚙️ Internal

- Updated `garde` (validation) to v0.20.
- Updated dependencies.

## 0.16.5

#### ⚙️ Internal

- Updated `garde` (validation) to v0.19.
- Updated dependencies.

## 0.16.4

#### 🚀 Updates

- Updated settings with `#[setting(default)]` or `#[serde(default)]` to be considered an "optional
  field" in the context of JSON schemas and TypeScript types.

## 0.16.3

#### 🐞 Fixes

- Fixed partial containers missing their comments.

## 0.16.2

#### 🚀 Updates

- Brought back the concept of `SchemaField`, as it solved some edge cases related to structs.

#### 🐞 Fixes

- Fixed missing container comments.

## 0.16.1

#### 🚀 Updates

- Added support for tuple based structs (newtypes).

## 0.16.0

#### 💥 Breaking

In preparation for v1, we've made a bunch of breaking changes. For the most part this is transparent
if using the macros, otherwise you'll need to update your schema implementations.

- Rewrote the `Schematic` trait (and indirectly the `Config` and `ConfigEnum` traits) from the
  ground up. The new API uses a builder pattern to construct the schema. This allows for all types
  to support names, descriptions, references, and more metadata. It also helps to avoid circular
  references.

  ```rust
  // Before
  impl Schematic for T {
    fn generate_schema() -> schematic::SchemaType {
      // Create the schema type
    }
  }

  // After
  impl Schematic for T {
    fn schema_name() -> Option<String> {
      None // Required for non-primitives
    }

    fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
      // Build the schema
      schema.build()
    }
  }
  ```

- Updated renderers with lifetimes, so that data from the generator can be borrowed correctly. If
  you're using the built-in renderers, everything should continue to work correctly.

  ```rust
  // Before
  impl SchemaRenderer<O> for T {
    fn render(
        &mut self,
        schemas: &IndexMap<String, Schema>,
        references: &HashSet<String>,
    ) -> RenderResult<O> {
      //
    }
  }

  // After
  impl<'gen> SchemaRenderer<'gen, O> for T<'gen> {
    fn render(
        &mut self,
        schemas: &'gen IndexMap<String, Schema>,
        references: &'gen HashSet<String>,
    ) -> RenderResult<O> {
      //
    }
  }
  ```

- Updated renderer methods to receive the schema as an immutable referenced argument. The schemas
  contains the name, description, and more.

  ```rust
  // Before
  impl SchemaRenderer<O> for T {
    fn render(&mut self, array: &ArrayType) -> RenderResult<O> {
      //
    }
  }

  // After
  impl<'gen> SchemaRenderer<'gen, O> for T<'gen> {
    fn render(&mut self, array: &ArrayType, schema: &Schema) -> RenderResult<O> {
      //
    }
  }
  ```

- Updated default value functions (handlers) to return a `Result`. This now aligns with the other
  handler functions.

  ```rust
  // Before
  fn default_count(ctx: &Context) -> Option<usize> {
      Some(10)
  }

  // After
  fn default_count(ctx: &Context) -> Result<Option<usize>, HandlerError> {
      Ok(Some(10))
  }
  ```

- Updated all handler functions, excluding validators, to return a `HandlerError`.
- Removed `SchemaField` and merged its functionality into `Schema`.

#### 🚀 Updates

- Added a `property_format` option to the TypeScript renderer.
- Added a `tracing` feature flag, that will wrap generated config methods with
  `#[tracing::instrument]`.
- Updated the macros to support `Box` for `#[setting(nested)]` struct fields.
- Updated the macro generated code to use `Box` in many places to reduce the size of enums and
  structs.
- Updated non-path based field types (tuples, arrays, etc) to support `Option`.

## 0.15.2

#### ⚙️ Internal

- Updated dependencies.
- Updated Rust to v1.78 (for development).

## 0.15.1

#### 🐞 Fixes

- Switched unit-only enums with a fallback to use "any of" instead of "one of", as the latter causes
  validation issues.

## 0.15.0

#### 🐞 Fixes

- Fixed unit-only enums with a fallback variant generating the wrong JSON schema and TypeScript
  types.

#### ⚙️ Internal

- Updated dependencies.
  - reqwest v0.11 -> v0.12
  - rustls v0.21 -> v0.22
- Updated Rust to v1.77.2 (for development).

## 0.14.5

#### 🚀 Updates

- Added a basic `ParseError` that can be used when implementing custom parsing via `TryFrom`,
  `FromStr`, serde, etc.

## 0.14.4

#### ⚙️ Internal

- Updated dependencies.

## 0.14.3

#### 🚀 Updates

- Added a `markdown_descriptions` option to the JSON Schema renderer. This will include a
  `markdownDescription` field in the schema output, which can be used by VSCode and other tools.
  This is a non-standard feature.

## 0.14.2

#### 🐞 Fixes

- Fixed some issues around JSON Schema `title` generation.

## 0.14.1

#### 🚀 Updates

- Added new JSON Schema renderer options:
  - `allow_newlines_in_description` - Allows newlines in descriptions, otherwise strips them.
    Defaults to `false`.
  - `mark_struct_fields_required` - Mark all non-option struct fields as required. Defaults to
    `true` for backwards compatibility.
  - `set_field_name_as_title` - Sets the field's name as the `title` of each schema entry. Defaults
    to `false`.

#### ⚙️ Internal

- Updated to Rust v1.76.
- Updated dependencies.

## 0.14.0

#### 💥 Breaking

- Removed `type_version_spec` and `type_warpgate` features (use the `schematic` feature on those
  crates instead).
- Renamed renderer related features:
  - `json_schema` -> `renderer_json_schema`
  - `template` -> `renderer_template`
  - `typescript` -> `renderer_typescript`
- Added a 4th boolean argument to validator functions, which denotes whether its validating the
  final config, or a partial config. This arg can be used to differentiate between the 2, change
  logic, or avoid validating.

#### 🚀 Updates

- Added 4 new validator functions:
  - `min_bytes` and `max_bytes`
  - `min_chars` and `max_chars`

#### ⚙️ Internal

- Updated `garde` (validation) to v0.18.
- Updated `miette` to v7.

## 0.13.7

#### 🚀 Updates

- Added `#[setting(alias)]` support (which maps to serde).

## 0.13.6

#### ⚙️ Internal

- Updated dependencies.

## 0.13.5

#### 🚀 Updates

- Added `type_indexmap` feature, that implements schematic types for `indexmap` values.

## 0.13.4

#### 🚀 Updates

- Added `#[setting(required)]` support for `Option`al settings.

#### ⚙️ Internal

- Updated `garde` (validation) to v0.17.
- Updated `version_spec` to v0.2.
- Updated `warpgate` to v0.9.

## 0.13.3

#### 🚀 Updates

- Added `JsonTemplateRenderer`, `JsoncTemplateRenderer`, `TomlTemplateRenderer`, and
  `YamlTemplateRenderer` for distinctness.
- Added `TemplateOptions.expand_fields` for expanding arrays and objects with an example item.

#### 🐞 Fixes

- Fixed nested configs receiving an environment variable when `env_prefix` is set.
- Fixed an issue where comments with bold markdown syntax was being rendered incorrectly.
- Fixed trailing commas for JSON template format.

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
