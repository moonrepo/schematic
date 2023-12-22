# Structs

The bulk of schematic is powered through the `Config` trait, the associated derive macro, and is
applied to structs. This macro helps to generate and automate the following:

- Generates a [partial struct](./partial.md), with all field values wrapped in `Option`.
- Provides [default value](./settings/default.md) and [environment variable](./settings/env.md)
  handling.
- Implements [merging](./settings/merge.md) and [validation](./settings/validate.md) logic.
- And other minor features, like [metadata](./context.md#metadata).

The struct that derives `Config` represents the _final state_, after all
[partial layers](./partial.md) have been merged, and default and environment variable values have
been applied. This means that all fields (settings) should _not_ be wrapped in `Option`, unless the
setting is truly optional (think nullable in the config file).

```rust
#[derive(Config)]
pub struct ExampleConfig {
	pub number: usize,
	pub string: String,
	pub boolean: bool,
	pub array: Vec<String>,
	pub optional: Option<String>,
}
```

> This pattern provides the optimal developer experience, as you can reference the settings as-is,
> without having to unwrap them, or use `match` or `if-let` statements!

## Attribute fields

The following fields are supported for the `#[config]` container attribute:

- `allow_unknown_fields` - Removes the serde `deny_unknown_fields` from the
  [partial struct](./partial.md). Defaults to `false`.
- `context` - Sets the struct to be used as the [context](./context.md). Defaults to `None`.
- `env_prefix` - Sets the prefix to use for [environment variable](./settings/env.md) mapping.
  Defaults to `None`.
- `file` - Sets a relative file path to use within error messages. Defaults to `None`.
- `serde` - A nested attribute that sets tagging fields for the [partial struct](./partial.md).
  Defaults to `None`.

```rust
#[derive(Config)]
#[config(allow_unknown_fields, env_prefix = "EXAMPLE_")]
pub struct ExampleConfig {
	// ...
}
```

And the following for serde compatibility:

- `rename`
- `rename_all`

## Serde support

By default the `Config` macro will apply
`#[serde(default, deny_unknown_fields, rename_all = "camelCase")]` to the
[partial struct](./partial.md). The `default` and `deny_unknown_fields` ensure proper parsing and
layer merging.

The `rename_all` field can be customized, and we also support the `rename` field, both via the
top-level `#[config]` attribute.

```rust
#[derive(Config)]
#[config(rename = "ExampleConfig", rename_all = "snake_case")]
struct Example {
	// ...
}
```

> These values can also be applied using `#[serde]`, which is useful if you want to apply them to
> the main struct as well, and not just the partial struct.
