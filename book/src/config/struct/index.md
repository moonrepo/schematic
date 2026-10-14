# Structs & enums

The [`Config`][config] trait can be derived for structs and enums.

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
	#[setting(nested)]
	Remote(HostConfig),
}
```

## Enum caveats

[`Config`][config] can only be derived for enums with tuple or unit variants, but not struct/named
variants. Why not struct variants? Because with this pattern, the enum acts like a union type. This
also allows for [`Config`][config] functionality, like partials, merging, and validation, to be
applied to the contents of each variant.

> If you'd like to support unit-only enums, you can use the [`ConfigEnum` trait](../enum/index.md)
> instead.

## Attribute fields

The following fields are supported for the `#[config]` container attribute:

- `allow_unknown_fields` - Removes the serde `deny_unknown_fields` from the
  [partial struct](../partial.md). Defaults to `false`.
- `context` - Sets the struct to be used as the [context](../context.md). Defaults to `None`.
- `env_prefix` - Sets the prefix to use for [environment variable](./env.md#container-prefixes)
  mapping. Defaults to `None`.
- `partial` - Forwards attributes to the [partial](../partial.md).

```rust
#[derive(Config)]
#[config(allow_unknown_fields, env_prefix = "EXAMPLE_")]
struct ExampleConfig {
	// ...
}
```

And the following for serde compatibility:

- `rename`
- `rename_all` - Has no default. Field names are used exactly as written unless this is set.
- `rename_all_fields` _(enum only)_ - Applies a casing to the fields of every variant.

## Serde support

By default the [`Config`][config] macro will apply the following `#[serde]` to the
[partial struct](../partial.md). The `default` and `deny_unknown_fields` ensure proper parsing and
layer merging.

```rust
#[serde(default, deny_unknown_fields)]
```

However, the `deny_unknown_fields` field can be customized, and we also support the `rename` and
`rename_all` fields, all via the top-level `#[config]` attribute.

```rust
#[derive(Config)]
#[config(allow_unknown_fields, rename = "ExampleConfig", rename_all = "snake_case")]
struct Example {
	// ...
}
```

> These values can also be applied using `#[serde]`, which is useful if you want to apply them to
> the main struct as well, and not just the partial struct.

### Enum tagging

Serde's tagging attributes are read straight off the `#[serde]` attribute and forwarded to the
partial, so `untagged`, `tag`, `content`, and `expecting` work as they normally would.

```rust
#[derive(Config)]
#[serde(untagged)]
enum Host {
	Local,
	#[setting(nested)]
	Remote(HostConfig),
}
```

The derive registers `serde` as a helper attribute, so this is accepted even when the type derives
only [`Config`][config] and not `Serialize` or `Deserialize`.

When a value matches none of an untagged enum's variants, the error lists why each one failed,
rather than serde's "data did not match any variant". Each starts with the path to the setting that
failed within the variant, and a nested untagged enum lists its own variants beneath the one that
holds it.

```
failed to parse as any variant of PartialHost:
- Local: expected null
- Remote: port: expected a 16-bit unsigned integer, found "abc"
```

> A unit-only enum is always externally tagged, no matter what you set. Marking one `untagged`
> would leave its variants deserializable only from `null`.

### Literal unions

A setting that accepts a few named values, or a value of another type, like
`"local" | "remote" | boolean`, is known as a literal union. Leave the enum externally tagged, so that
each unit variant is written as its name, and mark every variant that holds a value with
`#[serde(untagged)]`, so that it's written as that value alone.

```rust
#[derive(Config)]
#[serde(rename_all = "kebab-case")]
enum Cache {
	Local,
	#[setting(default)]
	Remote,
	#[serde(untagged)]
	Enabled(bool),
}

#[derive(Config)]
struct AppConfig {
	#[setting(nested)]
	pub cache: Cache,
}
```

With the above, `cache` accepts `"local"`, `"remote"`, `true`, or `false`, and generates the
TypeScript type `'local' | 'remote' | boolean`. The named values are matched first, then each untagged
variant in the order declared, which serde requires to come last. An untagged unit variant is
written as `null`.

To default to a value rather than a name, provide it on the untagged variant, like
`#[setting(default = true)]`. See [default values](./default.md#enum-variants).

When a value matches none of them, the error lists the named values together, followed by each
untagged variant.

```
failed to parse as any variant of PartialCache:
- expected one of "local", "remote", found "qux"
- enabled: expected a boolean, found "qux"
```

> Only mark the variants as `untagged`, not the enum. An untagged enum writes its unit variants as
> `null`, so they would no longer be named.

A literal union that holds only scalars, and should be settable through an
[environment variable](./env.md), can derive [`ConfigEnum`](../enum/untagged.md) instead, which
implements `FromStr` for it.

[config]: https://docs.rs/schematic/latest/schematic/trait.Config.html
