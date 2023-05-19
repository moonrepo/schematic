# schematic

> derive(Config)

Schematic is a light-weight, macro-based, layered serde configuration library, with built-in support
for merge strategies, validation rules, environment variables, and more!

- Load sources from the file system or secure URLs.
- Source layering that merge into a final configuration.
- Extend additional files through an annotated setting.
- Field-level merge strategies with built-in merge functions.
- Aggregated validation with built-in validate functions (provided by
  [garde](https://crates.io/crates/garde)).
- Environment variable parsing and overrides.
- Beautiful parsing and validation errors (powered by [miette](https://crates.io/crates/miette)).
- Supports JSON, TOML, and YAML via serde.

> This crate was built specifically for [moon](https://github.com/moonrepo/moon), and many of the
> design decisions are based around that project and its needs. Because of that, this crate is quite
> opinionated and won't change heavily.

## Usage

Define a struct and derive the `Config` trait.

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

The bulk of schematic is powered through the `Config` trait and the associated derive macro. This
macro helps to generate and automate the following:

- Generates a [partial configuration](#partials) struct, with all field values wrapped in `Option`.
- Provides [default value](#default-values) and [environment variable](#environment-variables)
  handling.
- Implements [merging](#merge-strategies) and [validation](#validation) logic.
- And other minor features, like [metadata](#metadata).

The struct that derives `Config` represents the _final state_, after all [partial layers](#partials)
have been merged, and default and environment variable values have been applied. This means that all
fields (settings) should _not_ be wrapped in `Option`, unless the setting is truly optional (think
nullable in the config file).

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

### Partials

A powerful feature of schematic is what we call partial configurations. These are a mirror of the
derived [configuration](#configuration), with all settings wrapped in `Option`, are prefixed with
`Partial`, and have common serde and derive attributes automatically applied.

For example, the `ExampleConfig` above would generate the following partial struct:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct PartialExampleConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub number: Option<usize>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub string: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub boolean: Option<bool>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub array: Option<Vec<String>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional: Option<String>,
}
```

So what are partials used for exactly? Partials are used for the entire parsing, layering,
extending, and merging process, instead of the base/final [configuration](#configuration).

When deserializing a source with serde, we utilize the partial config as the target type, because
not all fields are guaranteed to be present. This is especially true when merging multiple sources
together, as each source may only contain a subset of the final config. Each source represents a
layer to be merged.

Partials are also beneficial when serializing, as only settings with values will be written to the
source, instead of everything! A common complaint of serde's strictness.

As stated above, partials also handle the following:

- Defining [default values](#default-values) for settings.
- Inheriting [environment variable](#environment-variables) values.
- Merging partials with [strategy functions](#merge-strategies).
- Declaring [extendable sources](#extendable).

### Nested

[Configuration](#configuration) can easily be nested within other configuration using the
`#[setting(nested)]` attribute. Child configuration will be deeply merged and validated alongside
the parent.

```rust
#[derive(Config)]
pub struct ChildConfig {
	// ...
}

#[derive(Config)]
pub struct ParentConfig {
	#[setting(nested)]
	pub nested: ChildConfig,

	#[setting(nested)]
	pub optional_nested: Option<ChildConfig>,
}
```

The `#[setting(nested)]` attribute is required, as the macro will substitute the config struct with
its [partial struct](#partials) variant.

> Nested configuration can also be wrapped in collections, like `Vec` and `HashMap`.

### Contexts

Context is an important mechanism that allows for different [default values](#default-values),
[merge strategies](#merge-strategies), and [validation rules](#validation-rules) to be used, for the
_same_ configuration struct, depending on context!

To begin, a context is a struct with a default implementation.

```rust
#[derive(Default)]
struct ExampleContext {
	some_value: bool,
	another_value: usize,
}
```

Context must then be associated with a configuration through the `context` attribute field.

```rust
#[derive(Config)]
#[config(context = ExampleContext)]
pub struct ExampleConfig {
	// ...
}
```

And then passed to the `ConfigLoader.load_with_context` method.

```rust
let context = ExampleContext {
	some_value: true,
	another_value: 10,
};

let result = ConfigLoader::<ExampleConfig>::yaml()
	.url(url_to_config)?
	.load_with_context(&context)?;
```

Refer to the [default values](#default-values), [merge strategies](#merge-strategies), and
[validation rules](#validation-rules) sections for more information on how to use context.

### Metadata

[Configuration](#configuration) supports basic metadata for use within error messages through the
`#[config]` attribute. Right now we support a name, derived from the struct name or the serde
`rename` attribute field.

We also support a `file` field, which is typically the name of the configuration file that is being
loaded. This takes precedence over the name in error messages.

```rust
#[derive(Config)]
#[config(file = "example.json")]
pub struct ExampleConfig {
	// ...
}
```

Metadata can be accessed with the `META` constant.

```rust
ExampleConfig::META.name;
```

### Serde support

By default the `Config` macro will apply
`#[serde(default, deny_unknown_fields, rename_all = "camelCase")]` to the
[partial configuration](#partials). The `default` and `deny_unknown_fields` cannot be customized, as
they ensure proper parsing and layer merging.

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

- The first is on the [partial configuration](#partials), is defined with the `#[setting]`
  attribute, and is the first layer of the configuration to be merged.
- The second is on the [final configuration](#configuration) itself, and uses `Default` to generate
  the final value if none was provided. This acts more like a fallback.

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

When defining a custom merge function, the previous value, next value, and context are passed as
arguments, and the function must return an optional merged result. If `None` is provided, neither
value will be used.

Here's an example of the merge function above.

```rust
pub fn append_vec<T, C>(mut prev: Vec<T>, next: Vec<T>, context: &C) -> Result<Option<Vec<T>>, ConfigError> {
    prev.extend(next);

    Ok(Some(prev))
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
a path to a function to call. Furthermore, some functions are factories which can be called to
produce a validator.

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
