# Structs & enums

The `Config` trait can be derived for structs and enums.

```rust
#[derive(Config)]
struct AppConfig {
	pub base: String,
	pub port: usize,
	pub secure: bool,
	pub allowed_hosts: Vec<String>,
}

#[derive(Config)]
enum Host {
	Local,
	Remote(HostConfig),
}
```

## Enum caveats

`Config` can only be derived for enums with tuple or unit variants, but not struct/named variants.
Why not struct variants? Because with this pattern, the enum acts like a union type. This also
allows for `Config` functionality, like partials, merging, and validation, to be applied to the
contents of each variant.

> If you'd like to support unit-only enums, you can use the [`ConfigEnum` trait](../enum/index.md)
> instead.

## Attribute fields

The following fields are supported for the `#[config]` container attribute:

- `allow_unknown_fields` - Removes the serde `deny_unknown_fields` from the
  [partial struct](./partial.md). Defaults to `false`.
- `context` - Sets the struct to be used as the [context](./context.md). Defaults to `None`.
- `env_prefix` - Sets the prefix to use for
  [environment variable](./struct/env.md#container-prefixes) mapping. Defaults to `None`.
- `file` - Sets a relative file path to use within error messages. Defaults to `None`.
- `serde` - A nested attribute that sets tagging fields for the [partial](./partial.md). Defaults to
  `None`.

```rust
#[derive(Config)]
#[config(allow_unknown_fields, env_prefix = "EXAMPLE_")]
struct ExampleConfig {
	// ...
}
```

And the following for serde compatibility:

- `rename`
- `rename_all` - Defaults to `camelCase`.

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
