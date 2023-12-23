# Configuration

The bulk of Schematic is powered through the `Config` and `ConfigEnum` traits, and their associated
derive macro. These macros help to generate and automate the following (when applicable):

- Generates a [partial implementation](./partial.md), with all field values wrapped in `Option`.
- Provides [default value](./struct/default.md) and [environment variable](./struct/env.md)
  handling.
- Implements [merging](./struct/merge.md) and [validation](./struct/validate.md) logic.
- And other minor features, like [context & metadata](./context.md#metadata).

The struct or enum that derives `Config` represents the _final state_, after all
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
