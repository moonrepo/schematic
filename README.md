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
- Beautiful errors powered by the [miette](https://crates.io/crates/miette) crate.
- Supports JSON, TOML, and YAML via serde.

> This crate was built specifically for [moon](https://github.com/moonrepo/moon), and many of the
> design decisions are based around that project and its needs. Because of that, this crate is quite
> opinionated and won't change heavily.

## Usage

Define a struct and derive the `Config` trait. This struct represents the _final state_, after all
layers have been merged.

```rust
use schematic::Config;

#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000, env = "PORT")]
	port: usize,

	#[setting(default = true)]
	secure: bool,

	#[setting(default = vec!["localhost".into()])]
	allowed_hosts: Vec<String>,
}
```

Then load, parse, and validate the configuration from one or many sources. A source is either a file
path, secure URL, or code block.

```rust
use schematic::ConfigLoader;

let result = ConfigLoader::<AppConfig>::yaml()
	.code("secure: false")?
	.file(path_to_config)?
	.url(url_to_config)?
	.load()?;

result.config;
result.layers;
```

## Configuration

TODO

### Partials

TODO

### Nested

TODO

### Contexts

TODO

### Metadata

TODO

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
use schematic::ConfigError;

pub fn custom_parse(var: String) -> Result<ReturnValue, ConfigError> {
    do_parse().map_err(|e| ConfigError::Message(e.to_string()))
}
```

### Extendable

Configs can extend other configs, generating an accurate layer chain, via the `extend` attribute
field. Extended configs can either be a file path (relative from the current config) or a secure
URL. For example:

```yaml
extends:
  - "./another/file.yml"
  - "https://domain.com/some/other/file.yml"
```

When defining `extend`, we currently support 3 types of patterns. The first is with a single string,
which only allows a single file to be extended.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_string)]
	extends: Option<String>,
}
```

The second is with a list of strings, allowing multiple files to be extended. This is the YAML
example above.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_list)]
	extends: Option<Vec<String>>,
}
```

And lastly, supporting both a string or a list, using our built-in enum.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_from)]
	extends: Option<schematic::ExtendsFrom>,
}
```

> We suggest making this field optional, so that extending is not required by consumers!

### Merge strategies

A common requirement for configuration is to merge multiple sources/layers into a final result. By
default schematic will replace the previous value with the next value if the next value is `Some`,
but sometimes you want far more control, like shallow merging or deep merging collections.

This can be achieved with the `merge` attribute field, which requires a path to a function to call.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(merge = schematic::merge::append_vec)]
	allowed_hosts: Vec<String>,
}
```

> We provide a handful of built-in merge functions in the
> [`merge` module](https://docs.rs/schematic/latest/schematic/merge/index.html).

When defining a custom merge function, the previous and next values are passed as arguments, and the
function must return an optional merged result. If `None` is provided, either value will be used.

Here's an example of the merge function above.

```rust
pub fn append_vec<T>(mut prev: Vec<T>, next: Vec<T>) -> Option<Vec<T>> {
    prev.extend(next);

    Some(prev)
}
```

### Validation rules

What kind of configuration crate would this be without built-in validation? As such, we support it
as a first-class feature, with built-in validation rules provided by
[garde](https://crates.io/crates/garde).

In schematic, validation _does not_ happen as part of the serde parsing process, and instead happens
_after_ the [final configuration](#configuration) has been merged. This means we only validate the
end result, not [partial](#partials) values (which may be incorrect).

Validation can be applied on a per-setting basis with the `validate` attribute field, which requires
a path to a function. Furthermore, some functions are factories which can be called to produce a
validator.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(validate = schematic::validate::alphanumeric)]
	secret_key: String,

	#[setting(validate = schematic::validate::regex("^\.env"))]
	env_file: String,
}
```

> We provide a handful of built-in validation functions in the
> [`validate` module](https://docs.rs/schematic/latest/schematic/validate/index.html).

When defining a custom validate function, the value to check is passed as the first argument, the
current struct as the second, and the [context](#contexts) as the third. The `ValidateError` type
must be used for failures.

```rust
use schematic::ValidateError;

fn validate_string(
	value: &str,
	config: &AppConfig,
	context: &Context
) -> Result<(), ValidateError> {
	if !do_check(value) {
		return Err(ValidateError::new("Some failure message"));
	}

	Ok(())
}
```

If validating an item in a vector or collection, you can specifiy the nested path when erroring.
This is extremely useful when building error messages.

```rust
use schematic::Segment;

ValidateError::with_segments(
	"Some failure message",
	// [i].key
	[Segment::Index(i), Segment::Key(key.to_string())]
)
```

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

## Features

The following Cargo features are available:

### Parsing

- `json` (default) - Enables JSON.
- `toml` - Enables TOML.
- `yaml` - Enables YAML.

### Validation

- `valid_email` - Enables email validation with the `schematic::validate::email` function.
- `valid_url` - Enables URL validation with the `schematic::validate::url` and `url_secure`
  functions.
