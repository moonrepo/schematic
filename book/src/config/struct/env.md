# Environment variables

> Not supported for enums.

Settings can also inherit values from environment variables via the `#[setting(env)]` attribute
field. When using this, variables take the _highest_ precedence, and are merged as the last layer.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000, env = "PORT")]
	pub port: usize,
}
```

## Container prefixes

If you'd prefer to not define `env` for _every_ setting, you can instead define a prefix on the
containing struct using the `#[setting(env_prefix)]` attribute field. This will define an
environment variable for _all_ direct fields in the struct, in the format of "env prefix + field
name" in UPPER_SNAKE_CASE.

For example, the environment variable below for `port` is now `APP_PORT`.

```rust
#[derive(Config)]
#[config(env_prefix = "APP_")]
struct AppConfig {
	#[setting(default = 3000)]
	pub port: usize,
}
```

### Nested prefixes

Since `env_prefix` only applies to direct fields and not for nested/children structs, you'll need to
define `env_prefix` for each struct, and manually set the prefixes. Schematic _does not concatenate_
the prefixes between parent and child.

```rust
#[derive(Config)]
#[config(env_prefix = "APP_SERVER_")]
struct AppServerConfig {
	// ...
}

#[derive(Config)]
#[config(env_prefix = "APP_")]
struct AppConfig {
	#[setting(nested)]
	pub server: AppServerConfig,
}
```

## Parsing values

We also support parsing environment variables into the required type. For example, the variable may
be a comma separated list of values, or a JSON string.

The `#[setting(parse_env)]` attribute field can be used, which requires a path to a function to
handle the parsing, and receives the variable value as a single argument.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(env = "ALLOWED_HOSTS", parse_env = schematic::env::split_comma)]
	pub allowed_hosts: Vec<String>,
}
```

> We provide a handful of built-in parsing functions in the
> [`env` module](https://docs.rs/schematic/latest/schematic/env/index.html).

## Parse function

You can also define your own function for parsing values out of environment variables.

When defining a custom parse function, the variable value is passed as the 1st argument, and a
`Result<Option>` must be returned. A `None` value can be returned, which will fallback to the
previous or default value.

```rust
use schematic::ConfigError;

pub fn custom_parse(var: String) -> Result<Some<ReturnValue>, ConfigError> {
	do_parse()
		.map(|v| Some(v))
		.map_err(|e| ConfigError::Message(e.to_string()))
}
```

If parsing fails, you must return a
[`ConfigError`](https://docs.rs/schematic/latest/schematic/enum.ConfigError.html) with a failure
message.
