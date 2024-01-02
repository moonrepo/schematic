# Default values

In Schematic, there are 2 forms of default values:

- The first is applied through the [partial configuration](../partial.md), is defined with the
  `#[setting]` attribute, and is the first layer to be merged.
- The second is on the [final configuration](../index.md) itself, and uses the `Default` trait to
  generate the final value if none was provided. This acts more like a fallback.

To define a default value, use the `#[setting(default)]` attribute. The `default` attribute field is
used for declaring primitive values, like numbers, strings, and booleans, but can also be used for
array and tuple literals, as well as function (mainly for `from()`) and macros calls.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = "/")]
	pub base: String,

	#[setting(default = 3000)]
	pub port: usize,

	#[setting(default = true)]
	pub secure: bool,

	#[setting(default = vec!["localhost".into()])]
	pub allowed_hosts: Vec<String>,
}
```

For enums, the `default` field takes no value, and simply marks which variant to use as the default.

```rust
#[derive(Config)]
enum Host {
	#[setting(default)]
	Local,
	Remote(HostConfig),
}
```

## Handler function

If you need more control or need to calculate a complex value, you can pass a reference to a
function to call. This function receives the [context](../context.md) as the first argument, and can
return an optional value. If `None` is returned, the `Default` value will be used instead.

```rust
fn find_unused_port(ctx: &Context) -> Option<usize> {
	let port = do_find();
	Some(port)
}

#[derive(Config)]
struct AppConfig {
	#[setting(default = find_unused_port)]
	pub port: usize,
}
```

### Context handling

If you're not using [context](../context.md), you can use `()` as the context type, or rely on
generic inferrence.

```rust
fn using_unit_type(_: &()) -> Option<usize> {
	// ...
}

fn using_generics<C>(_: &C) -> Option<usize> {
	// ...
}
```
