# Default values

In Schematic, there are 2 forms of default values:

- The first is applied through the [partial configuration](../partial.md), is defined with the
  `#[setting]` attribute, and is the first layer to be merged.
- The second is on the [final configuration](../index.md) itself, and uses the `Default` trait to
  generate the final value if none was provided. This acts more like a fallback.

To define a default value, use the `#[setting(default)]` attribute. The `default` attribute field
accepts primitive values, like numbers, strings, and booleans, as well as array, tuple, and struct
literals, paths that name a value, and function and macro calls.

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

	#[setting(default = LevelFilter::Debug)]
	pub log_level: LevelFilter,

	#[setting(default = Retry { attempts: 3, backoff: 500 })]
	pub retry: Retry,
}
```

### Paths: value or function?

A bare path is ambiguous — `LevelFilter::Debug` names a value, while `find_unused_port` names a
[handler function](#handler-function) to call. Nothing at compile time can tell them apart, so
Schematic goes by Rust's naming conventions:

| Last segment of the path        | Treated as       | Example                  |
| ------------------------------- | ---------------- | ------------------------ |
| Starts uppercase                | A value          | `LevelFilter::Debug`     |
| Starts lowercase                | A handler function | `find_unused_port`     |

That covers enum variants, unit structs, and constants on one side, and functions on the other,
since a name breaking those conventions would already be earning a compiler lint.

> A handler function that must keep a non-conventional name can be reached through a `snake_case`
> wrapper, or called directly with `default = my_wrapper()` when it needs no context.

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
fn find_unused_port(ctx: &Context) -> DefaultValueResult<usize> {
	let port = do_find()?;

	Ok(Some(port))
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
fn using_unit_type(_: &()) -> DefaultValueResult<usize> {
	// ...
}

fn using_generics<C>(_: &C) -> DefaultValueResult<usize> {
	// ...
}
```
