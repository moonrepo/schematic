# Changelog

## Unreleased

#### 💥 Breaking

- Renamed `ConfigMeta` trait to `Meta`.

#### 🚀 Updates

- Added `ConfigEnum` trait that the `ConfigEnum` derive macro implements.
  - Trait contains `META`.
  - Trait provides a `variants()` method.
- Added `fields` to `Meta` trait.
  - Updated `Config` trait to implement fields.
- Added TypeScript declaration/type generation.
  - Behind a new `typescript` flag.
  - Code can be generated with the `schematic::typescript::TypeScriptGenerator`.

#### ⚙️ Internal

- Refactored `derive_enum` maro (should be backwards compatible).
