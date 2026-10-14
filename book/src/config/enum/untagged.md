# Untagged variants

A setting that accepts a few named values, or a value of another type, like `"auto" | "off" | number`,
is known as a literal union. To model one, mark each variant that holds a value with
`#[serde(untagged)]`, so that serde reads it as that value alone.

```rust
derive_enum!(
	#[derive(ConfigEnum, Default)]
	enum Threads {
		#[default]
		Auto,
		Off,
		#[serde(untagged)]
		Count(u32),
	}
);
```

Parsing from a string matches the named values first, then attempts each untagged variant in the
order declared, using the inner type's `FromStr`. Formatting writes the inner value with `Display`.
This is what allows the setting to be read from an
[environment variable](../struct/env.md).

```rust
let auto = Threads::from_str("auto")?; // Threads::Auto
let count = Threads::from_str("8")?; // Threads::Count(8)
```

However, this pattern does have a few caveats:

- An untagged variant must be a tuple variant with a single field.
- The field type must implement `Default`, `Display`, and `FromStr`.
- Untagged variants are attempted before the [fallback variant](./fallback.md), if there is one.
- The input is parsed as written, without the `before_parse` transform, which only exists to
  normalize named values.
- serde requires untagged variants to come after every other variant.

> A literal union with a variant that holds a nested configuration can derive
> [`Config`](../struct/index.md#literal-unions) instead.
