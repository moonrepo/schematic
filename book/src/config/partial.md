# Partials

A powerful feature of Schematic is what we call partial configurations. These are a mirror of the
derived [`Config` struct](./struct/index.md) or [`Config` enum](./struct/index.md), with all
settings wrapped in `Option`, the item name prefixed with `Partial`, and have common serde and
derive attributes automatically applied.

For example, the `ExampleConfig` from the [first chapter](../config/index.md) would generate the
following partial struct:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PartialExampleConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub number: Option<usize>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub string: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub boolean: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub array: Option<Vec<String>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional: Option<String>,
}
```

So what are partials used for exactly? Partials are used for the entire parsing, layering,
extending, and merging process, and ultimately become the [final configuration](./index.md).

When deserializing a source with serde, we utilize the partial config as the target type, because
not all fields are guaranteed to be present. This is especially true when merging multiple sources
together, as each source may only contain a subset of the final config. Each source represents a
layer to be merged.

Partials are also beneficial when serializing, as only settings with values will be written to the
source, instead of everything! A common complaint of serde's strictness.

As stated above, partials also handle the following:

- Defining [default values](./struct/default.md) for settings.
- Inheriting [environment variable](./struct/env.md) values.
- Merging partials with [strategy functions](./struct/merge.md).
- Validating current values with [validate functions](./struct/validate.md).
- Declaring [extendable sources](./struct/extend.md).

## Partial attribute forwarding

Attributes can be forwarded to the generated partial struct using the `#[config(partial())]`
attribute on structs and enums.

```rust
#[derive(Config)]
#[config(partial(derive(derive_more::AsRef)))]
struct ExampleConfig {
	//
}
```

Fields attributes can be forwarded using `#[setting(partial())]`.

```rust
#[derive(Config)]
#[config(partial(derive(derive_more::AsRef)))]
struct ExampleConfig {
	#[setting(partial(as_ref))]
	port: usize
}
```
