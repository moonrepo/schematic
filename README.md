# Schematic

> derive(Config)

TODO

- Load sources from the file system or secure URLs.
- Source layering that merge into a final configuration.
- Extend additional files through an annotated setting.
- Field-level merge strategies with built-in merge functions.
- Aggregated validation with built-in validate functions (provided by
  [garde](https://crates.io/crates/garde)).
- Environment variable parsing and overrides.

> This crate was built specifically for [moon](https://github.com/moonrepo/moon), and many of the
> design decisions are based around that project and its needs. Because of that, this crate is quite
> opinionated and won't change heavily.

## Configuration

### Partials

### Nested configuration

### Contexts

### Metadata

### Serde support

By default the `Config` macro will apply
`#[serde(default, deny_unknown_fields, rename_all = "camelCase")]` to the
[partial config](#partials). The `default` and `deny_unknown_fields` cannot be customized, as they
ensure proper parsing and layer merging.

However, the `rename_all` field can be customized, and we also support the `rename` field, both via
the top-level `#[config]` attribute.

```rust
#[derive(Config)]
#[config(rename = "ExampleConfig", rename_all = "snake_case")]
struct Example {
	// ...
}
```

> The `rename` field will also update the [metadata](#metadata) name.

## Settings

Settings are the individual fields/members of a configuration struct, and can be annotated with the
optional `#[setting]` attribute.

### Default values

In schematic, there are 2 forms of default values:

- The first is on the [partial config](#partials), is defined with the `#[setting]` attribute, and
  is the first layer of the configuration to be merged.
- The second is on the [final config](#configuration) itself, and uses `Default` to generate the
  final value if none was provided. This acts more like a fallback.

This section will talk about the `#[setting]` attribute, and the supported `default`, `default_str`,
and `default_fn` attribute fields.

The `default` attribute field is used for declaring primitive values, like numbers and booleans. It
can also be used for array and tuple literals, as well as function (mainly for `from()`) and macros
calls.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000)]
	port: usize,

	#[setting(default = true)]
	secure: bool,

	#[setting(default = vec!["localhost".into()])]
	allowed_hosts: Vec<String>,
}
```

For string literals, the `default_str` attribute field should be used instead.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default_str = "/")]
	base: String,
}
```

And lastly, if you need more control or need to calculate a complex value, you can use the
`default_fn` attribute field, which requires a path to a function to call.

This function receives the [context](#contexts) as the first argument (use `()` or generics if you
don't have context), and can return an optional value. If `None` is returned, the `Default` value
will be used instead.

```rust
fn find_unused_port(ctx: &Context) -> Option<usize> {
	let port = do_find();
	Some(port)
}

#[derive(Config)]
struct AppConfig {
	#[setting(default_fn = find_unused_port)]
	port: usize,
}
```

### Environment variables

Settings can also inherit values from environment variables via the `env` attribute field. When
using this, variables take the _highest_ precedence, and are merged as the last layer.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000, env = "PORT")]
	port: usize,
}
```

We also support parsing environment variables into the required type. For example, the variable may
be a comma separated list of values, or a JSON string.

The `parse_env` attribute field can be used, which requires a path to a function to handle the
parsing, and receives the variable value as a single argument.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(env = "ALLOWED_HOSTS", parse_env = schematic::env::split_comma)]
	allowed_hosts: Vec<String>,
}
```

> We provide a handful of built-in parsing functions in the
> [`env` module](https://docs.rs/schematic/latest/schematic/env/index.html).

When defining a custom parse function, you should return an error with `ConfigError::Message` if
parsing fails.

```rust
pub fn custom_parse(var: String) -> Result<ReturnValue, ConfigError> {
    do_parse().map_err(|e| ConfigError::Message(e.to_string()))
}
```

### Extendable

### Merge strategies

### Validation rules

### Serde support

The `rename` and `skip` attribute fields are currently supported and will apply a `#[serde]`
attribute to the [partial](#partials) setting.

```rust
#[derive(Config)]
struct Example {
	#[setting(rename = "type")]
	type_of: SomeEnum,
}
```
