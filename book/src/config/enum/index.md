# Unit-only enums

Configurations typically use enums to support multiple values within a specific
[setting](../settings.md). To simplify this process, and to provide streamlined interoperability
with [`Config`][config], we offer a
[`ConfigEnum`](https://docs.rs/schematic/latest/schematic/trait.ConfigEnum.html) trait and macro
that can be derived for enums with unit-only variants.

```rust
#[derive(ConfigEnum)]
enum LogLevel {
	Info,
	Error,
	Debug,
	Off
}
```

When paired with [`Config`][config], it'll look like:

```rust
#[derive(Config)]
struct AppConfig {
	pub log_level: LogLevel
}
```

This enum will generate the following implementations:

- Provides a static
  [`T::variants()`](https://docs.rs/schematic/latest/schematic/trait.ConfigEnum.html#tymethod.variants)
  method, that returns a list of all variants. Perfect for iteration.
- Implements `FromStr` and `TryFrom` for parsing from a string.
- Implements `Display` for formatting into a string.

## Attribute fields

The following fields are supported for the `#[config]` container attribute:

- `before_parse` - Transform the variant string value before parsing. Supports `lowercase` or
  `UPPERCASE`.

```rust
#[derive(ConfigEnum)]
#[config(before_parse = "UPPERCASE")]
enum ExampleEnum {
	// ...
}
```

And the following for serde compatibility:

- `rename`
- `rename_all` - Defaults to `kebab-case`.

### Variants

The following fields are supported for the `#[variant]` variant attribute:

- `fallback` - Marks the variant as the [fallback](./fallback.md).
- `value` - Overrides (explicitly sets) the string value used for parsing and formatting. This is
  similar to serde's `rename`.

And the following for serde compatibility:

- `alias`
- `rename`

## Deriving common traits

All enums (not just unit-only enums) typically support the same derived traits, like `Clone`, `Eq`,
etc. To reduce boilerplate, we offer a
[`derive_enum!`](https://docs.rs/schematic/latest/schematic/macro.derive_enum.html) macro that will
apply these traits for you.

```rust
derive_enum!(
	#[derive(ConfigEnum)]
	enum LogLevel {
		Info,
		Error,
		Debug,
		Off
	}
);
```

This macro will inject the following attributes:

```rust
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
```

[config]: https://docs.rs/schematic/latest/schematic/trait.Config.html
