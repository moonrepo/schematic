# Nesting

[`Config` structs](./index.md) can easily be nested within other `Config`s using the
`#[setting(nested)]` attribute. Children will be deeply merged and validated alongside the parent.

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
	#[config(nested)]
	Variant(ChildConfig),
}
```

The `#[setting(nested)]` attribute is required, as the macro will substitute `Config` with its
[partial](./partial.md) implementation.

> Nested values can also be wrapped in collections, like `Vec` and `HashMap`. However, these are
> tricky to support and may not work in all situations!

## Bare structs

For structs that _do not_ implement the `Config` trait, you can use them as-is without the
`#[setting(nested)]` attribute. When using bare structs, be aware that all of the functionality
provided by our `Config` trait is not available, like merging and validation.

```rust
struct BareConfig {
	// ...
}

#[derive(Config)]
pub struct ParentConfig {
	pub nested: BareConfig,
}
```
