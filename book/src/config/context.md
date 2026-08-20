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

Alongside the configuration itself, the derive records a little metadata about each
[setting](./settings.md), reachable with
[`Config::settings()`](https://docs.rs/schematic/latest/schematic/trait.Config.html#method.settings).
It returns a map keyed by the serde name of each setting, or by position for unnamed ones.

```rust
for (name, setting) in ExampleConfig::settings() {
	println!("{name}: {}", setting.type_alias);

	if let Some(key) = &setting.env_key {
		println!("  reads {key}");
	}

	if let Some(nested) = &setting.nested {
		println!("  has {} nested settings", nested.len());
	}
}
```

Each entry carries the setting's `type_alias` (the Rust type as written), its `env_key`, and a
`nested` map when the setting holds another [nested config](./nested.md).

> Only an explicit `#[setting(env)]` populates `env_key`. A key derived from an
> [`env_prefix`](./struct/env.md#container-prefixes) depends on the prefix in effect at runtime, so
> it isn't known here.
