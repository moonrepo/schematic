# Context

Context is an important mechanism that allows for different [default values](./struct/default.md),
[merge strategies](./struct/merge.md), and [validation rules](./struct/validate.md) to be used, for
the _same_ configuration struct, depending on context!

To begin, a context is a struct with a default implementation.

```rust
#[derive(Default)]
struct ExampleContext {
	pub some_value: bool,
	pub another_value: usize,
}
```

Context must then be associated with a
[`Config`](https://docs.rs/schematic/latest/schematic/trait.Config.html) derived struct through the
`context` attribute field.

```rust
#[derive(Config)]
#[config(context = ExampleContext)]
struct ExampleConfig {
	// ...
}
```

And then passed to the
[`ConfigLoader::load_with_context()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.load_with_context)
method.

```rust
let context = ExampleContext {
	some_value: true,
	another_value: 10,
};

let result = ConfigLoader::<ExampleConfig>::new()
	.url(url_to_config)?
	.load_with_context(&context)?;
```

> Refer to the [default values](./struct/default.md), [merge strategies](./struct/merge.md), and
> [validation rules](./struct/validate.md) sections for more information on how to use context.

## Metadata

[`Config`](./index.md) supports basic metadata for use within error messages. Right now we support a
name, derived from the struct/enum name or the serde `rename` attribute field.

Metadata can be accessed with the `META` constant.

```rust
ExampleConfig::META.name;
```
