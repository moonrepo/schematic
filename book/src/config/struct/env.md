# Environment variables

> Requires the `env` Cargo feature, which is enabled by default.

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
containing struct using the `#[config(env_prefix)]` container attribute field. This will define an
environment variable for _all_ direct fields in the struct, in the format of "env prefix + field
name" in uppercase.

For example, the environment variable below for `port` is now `APP_PORT`.

```rust
#[derive(Config)]
#[config(env_prefix = "APP_")]
struct AppConfig {
	#[setting(default = 3000)]
	pub port: usize,
}
```

A derived key is the field's Rust name (or its explicit `rename`) uppercased, and nothing more. It
is deliberately not reshaped by `rename_all`, so changing how a setting is spelled in a config file
never moves its environment variable.

An explicit `#[setting(env)]` is absolute. A prefix is never applied to it, and the two cannot be
combined on the same setting.

### Nested prefixes

`env_prefix` only applies to direct fields, not to nested children, and prefixes are _not_
concatenated between parent and child. Each struct declares its own.

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

A parent can add a prefix for a nested child to read, with `env_prefix` on the field itself. The
child must still declare an `env_prefix` of its own, as that is what opts its fields into being
derived at all. Without one, the parent's prefix would have no effect, so it fails to compile.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(nested, env_prefix = "OVERRIDE_")]
	pub server: AppServerConfig,
}
```

> The parent's prefix is only known at runtime, so it isn't reflected in the
> [schema](../../schema/index.md). A derived key appears in a generated
> [config template](../../schema/generator/template.md) and in
> [`Config::settings()`](https://docs.rs/schematic/latest/schematic/trait.Config.html#method.settings)
> with the child's own prefix, which always works.

## Enums

The fields of a [named variant](./index.md#enum-variants) read environment variables like a
struct's, with `env`, `parse_env`, and an `env_prefix` on the enum, which derives a key for each
field.

```rust
#[derive(Config)]
#[config(env_prefix = "AUTH_")]
#[serde(tag = "type", rename_all = "snake_case")]
enum Auth {
	#[setting(default)]
	None,
	Token {
		// AUTH_TOKEN
		token: String,
	},
	Basic {
		// AUTH_USER
		user: String,
		#[setting(env = "AUTH_PASSWORD")]
		password: String,
	},
}
```

A variable can't pick a variant, it only fills one in. The variables are read once a source, or
the default, has picked the variant, and they're merged over its fields as the last layer, as for a
struct. So with `type = "basic"` in a file, `AUTH_TOKEN` does nothing. Variants may share a derived
key, like a `path` in each, as only one is ever picked.

A nested config that a variant holds reads its own variables, whether the variant is named or holds
it as a value.

> The variables of a variant aren't a layer of their own, so `ConfigLoadResult.layers` doesn't
> include them, as with the items of a nested collection.

A parent's `#[setting(nested, env_prefix)]` doesn't reach an enum, since its variables are only read
once a variant has been picked, without the parent. Setting one on an enum fails to compile.

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

`parse_env` needs a variable to read, so it requires either an `env` on the setting, or an
`env_prefix` on the container. It's a derive-time error otherwise.

## Parse handler function

You can also define your own function for parsing values out of environment variables.

When defining a custom `parse_env` function, the variable value is passed as the 1st argument. A
`None` value can be returned, which will fallback to the previous or default value.

```rust
pub fn custom_parse(var: String) -> ParseEnvResult<ReturnValue> {
	do_parse()
		.map(|v| Some(v))
		.map_err(|e| HandlerError::new(e.to_string()))
}

#[derive(Config)]
struct ExampleConfig {
	#[setting(env = "FIELD", parse_env = custom_parse)]
	pub field: String,
}
```
