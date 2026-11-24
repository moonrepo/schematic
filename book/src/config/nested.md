# Nesting

[`Config` structs](./struct/index.md) can easily be nested within other [`Config`][config]s using
the `#[setting(nested)]` attribute. Children will be deeply merged and validated alongside the
parent.

```rust
#[derive(Config)]
struct ChildConfig {
	// ...
}

#[derive(Config)]
struct ParentConfig {
	#[setting(nested)]
	pub nested: ChildConfig,

	#[setting(nested)]
	pub optional_nested: Option<ChildConfig>,
}

#[derive(Config)]
enum ParentEnum {
	#[setting(nested)]
	Variant(ChildConfig),
}
```

The `#[setting(nested)]` attribute is required, as the macro will substitute [`Config`][config] with
its [partial](./partial.md) implementation.

Nested values can also be wrapped in collections, like `Vec` and `HashMap`. Each item is finalized
and validated as a nested config, but the collection is replaced when merged, rather than merging
its items. To merge a map of configs by key, use the
[`merge_partials` strategy](./struct/merge.md#nested-maps).

A `#[setting(nested)]` on a variant applies to every value it holds. To hold a nested config
alongside plain values, mark the value itself instead. Its config is merged, finalized, and
validated as nested, while the other values are replaced, like any other setting.

```rust
#[derive(Config)]
enum ParentEnum {
	Both(#[setting(nested)] ChildConfig, bool),
}
```

## Flattening

A nested config can be flattened with `#[setting(flatten, nested)]` (or serde's own
`#[serde(flatten)]`), so its settings are written alongside the parent's own, rather than under the
field's name. A map can be flattened too, which collects every key the other settings don't.

```rust
#[derive(Config)]
struct ParentConfig {
	pub name: String,

	#[setting(flatten, nested)]
	pub server: ServerConfig,

	#[setting(flatten, nested)]
	pub plugins: HashMap<String, PluginConfig>,
}
```

A config rejects keys that aren't its settings, unless `#[config(allow_unknown_fields)]`, and serde
can only do so for a flattened struct, which takes its keys from the parent's. So:

- A config with a flattened map accepts unknown keys, as the map collects them, and rejecting them
  would reject every key of the map.
- A flattened config that flattens another can't be loaded unless both allow unknown fields, as it
  reads the parent's keys without taking them, so the parent would reject them. Deriving one fails
  to compile, naming the fix, rather than failing to load every setting it holds.

```rust
#[derive(Config)]
#[config(allow_unknown_fields)]
struct ServerConfig {
	#[setting(flatten, nested)]
	pub tls: TlsConfig,
}
```

A value in a flattened map that fails to deserialize fails the load, prefixed with its key, like
``foo.version: invalid type: integer `2`, expected a string``. Serde discards the error of any other
flattened setting, as it may be absent, so a flattened struct holding an invalid value is loaded as
if it held nothing.

## Bare structs

For structs that _do not_ implement the [`Config`][config] trait, you can use them as-is without the
`#[setting(nested)]` attribute. When using bare structs, be aware that all of the functionality
provided by our [`Config`][config] trait is not available, like merging and validation.

```rust
struct BareConfig {
	// ...
}

#[derive(Config)]
pub struct ParentConfig {
	pub nested: BareConfig,
}
```

[config]: https://docs.rs/schematic/latest/schematic/trait.Config.html
