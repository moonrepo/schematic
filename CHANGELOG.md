# Changelog

## Unreleased

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
  - Behind a new `typescript` flag.
  - Code can be generated with the `schematic::typescript::TypeScriptGenerator`.
- Updated partials to inherit `#[allow]`, `#[warn]`, and `#[deprecated]` attributes.

#### ⚙️ Internal

- Refactored `derive_enum` maro (should be backwards compatible).
