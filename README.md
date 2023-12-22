# schematic

> derive(Config)

Schematic is a light-weight, macro-based, layered serde configuration and schema library, with
built-in support for merge strategies, validation rules, environment variables, and more!

- Supports JSON, TOML, and YAML based configs via serde.
- Load sources from the file system or secure URLs.
- Source layering that merge into a final configuration.
- Extend additional files through an annotated setting.
- Field-level merge strategies with built-in merge functions.
- Aggregated validation with built-in validate functions (provided by
  [garde](https://crates.io/crates/garde)).
- Environment variable parsing and overrides.
- Beautiful parsing and validation errors (powered by [miette](https://crates.io/crates/miette)).
- Generates schemas that can be rendered to TypeScript types, JSON schemas, and more!

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

Then load, parse, merge, and validate the configuration from one or many sources. A source is either
a file path, secure URL, or code block.

```rust
use schematic::{ConfigLoader, Format};

let result = ConfigLoader::<AppConfig>::new()
	.code("secure: false", Format::Yaml)?
	.file("path/to/config.yml")?
	.url("https://ordomain.com/to/config.yaml")?
	.load()?;

result.config;
result.layers;
```

> The format for files and URLs are derived from the trailing extension.

## Configuration

## Configuration enums

Configurations typically use enums to handle value variations of a specific [setting](#settings). To
simplify this process, we offer a `ConfigEnum` macro/trait that can be derived for enums with
unit-only variants.

```rust
#[derive(ConfigEnum)]
enum LogLevel {
	Info,
	Error,
	Debug,
	Off
}
```

This enum will generate the following implementations:

- Provides a static `variants` method, that returns a list of all variants. Perfect for iteration.
- Implements `FromStr` and `TryFrom` for parsing from a string.
- Implements `Display` for formatting into a string.

The string value/format is based on the variant name, and is converted to kebab-case by default.
This can be customized with the `#[serde(rename_all = "kebab-case")]` attribute, which keeps
consistency with serde's handling.

### Fallback variant

Although `ConfigEnum` only supports unit variants, we do support a catch-all variant known as the
"fallback variant", which can be defined with `#[variant(fallback)]`. Fallback variants are
primarily used when parsing from a string, and will be used if no other variant matches.

However, this pattern does have a few caveats:

- Only 1 fallback variant can be defined.
- The fallback variant must be a tuple variant with a single field.
- The field type can be anything and we'll attempt to convert it with `try_into()`.
- The fallback inner value _is not_ casing formatted based on serde's `rename_all`.

```rust
#[derive(ConfigEnum)]
enum Value {
	Foo,
	Bar,
	Baz
	#[variant(fallback)]
	Other(String)
}
```

### Common derives

Furthermore, all enums (not just unit enums) typically support the same derived traits, like
`Clone`, `Eq`, etc. To reduce boilerplate, we offer a `derive_enum!` macro that will apply these
traits for you.

```rust
derive_enum!(
	#[derive(ConfigEnum, Default)]
	enum LogLevel {
		Info,
		Error,
		Debug,
		#[default]
		Off
	}
);
```

This macro will inject the following attributes:

```rust
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
```

## Settings

Settings are the individual fields/members of a configuration struct, and can be annotated with the
optional `#[setting]` attribute.

### Default values

> Structs only.

In schematic, there are 2 forms of default values:

- The first is on the [partial configuration](#partials), is defined with the `#[setting]`
  attribute, and is the first layer of the configuration to be merged.
- The second is on the [final configuration](#configuration) itself, and uses `Default` to generate
  the final value if none was provided. This acts more like a fallback.

This section will talk about the `#[setting]` attribute and `default`. The `default` attribute field
is used for declaring primitive values, like numbers and booleans. It can also be used for array and
tuple literals, as well as function (mainly for `from()`) and macros calls.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = "/")]
	base: String,

	#[setting(default = 3000)]
	port: usize,

	#[setting(default = true)]
	secure: bool,

	#[setting(default = vec!["localhost".into()])]
	allowed_hosts: Vec<String>,
}

#[derive(Config)]
enum Host {
	#[setting(default)]
	Local,
	Remote(HostConfig),
}
```

> Enums only support `#[setting(default)]`, which denotes that variant as the default. It does not
> support setting values for the variant itself, or its inner tuple fields.

If you need more control or need to calculate a complex value, you can pass a reference to a
function to call. This function receives the [context](#contexts) as the first argument (use `()` or
generics if you don't have context), and can return an optional value. If `None` is returned, the
`Default` value will be used instead.

```rust
fn find_unused_port(ctx: &Context) -> Option<usize> {
	let port = do_find();
	Some(port)
}

#[derive(Config)]
struct AppConfig {
	#[setting(default = find_unused_port)]
	port: usize,
}
```

### Environment variables

> Structs only.

Settings can also inherit values from environment variables via the `env` attribute field. When
using this, variables take the _highest_ precedence, and are merged as the last layer.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000, env = "PORT")]
	port: usize,
}
```

#### Container prefixes

If you'd prefer to not define `env` for _every_ setting, you can instead define a prefix on the
containing struct using the `env_prefix` attribute field. This will define an environment variable
for _all_ non-nested fields in the struct, in the format of "env prefix + field name" in
UPPER_SNAKE_CASE.

For example, the environment variable below is now `APP_PORT`.

```rust
#[derive(Config)]
#[config(env_prefix = "APP_")]
struct AppConfig {
	#[setting(default = 3000)]
	port: usize,
}
```

#### Parsing values

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
parsing fails. A `None` value can also be returned, which will fallback to the previous or default
value.

```rust
use schematic::ConfigError;

pub fn custom_parse(var: String) -> Result<Some<ReturnValue>, ConfigError> {
	do_parse()
		.map(|v| Some(v))
		.map_err(|e| ConfigError::Message(e.to_string()))
}
```

### Extendable

> Structs only.

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

#[derive(Config)]
enum Projects {
	#[setting(merge = schematic::merge::append_vec)]
	List(Vec<String>),
	// ...
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
_for each_ [partial configuration](#partials) to be merged.

Validation can be applied on a per-setting basis with the `validate` attribute field, which requires
a path to a function to call.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(validate = schematic::validate::alphanumeric)]
	secret_key: String,

	#[setting(validate = schematic::validate::regex("^\.env"))]
	env_file: String,
}
```

Or on a per-variant basis when using an enum.

```rust
#[derive(Config)]
enum Projects {
	#[setting(validate = schematic::validate::min_length(1))]
	List(Vec<String>),
	// ...
}
```

> We provide a handful of built-in validation functions in the
> [`validate` module](https://docs.rs/schematic/latest/schematic/validate/index.html). Furthermore,
> some functions are factories which can be called to produce a validator.

When defining a custom validate function, the value to check is passed as the first argument, the
current partial as the second, and the [context](#contexts) as the third. The `ValidateError` type
must be used for failures.

```rust
use schematic::ValidateError;

fn validate_string(
	value: &str,
	partial: &PartialAppConfig,
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
use schematic::PathSegment;

ValidateError::with_segments(
	"Some failure message",
	// [i].key
	[PathSegment::Index(i), PathSegment::Key(key.to_string())]
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

## Generators

Schematic provides a schema modeling layer that defines the shape of types, which all configuration
and enums implement. These schemas can then be passed to a generator, which renders the schema into
a specific format, and writes the result to a file.

```rust
use schematic::{schema, renderers};

fn main() {
	let mut generator = schema::SchemaGenerator::default();
	generator.add::<ConfigOne>();
	generator.add::<ConfigTwo>();
	generator.add::<EnumThree>();
	generator.add::<OtherWithSchemas>();
}
```

> Added types will recursively add all nested schemas, so you only need to add the root types, and
> not everything!

### JSON schemas

- Enabled with the `json_schema` feature.
- The last schema to be added to the generator will be the root document, while all previous schemas
  will be definitions/references.

```rust
generator.generate(
	output_dir.join("schema.json"),
	schema::json_schema::JsonSchemaRenderer::default(),
);
```

### TypeScript types

- Enabled with the `typescript` feature.
- Each schema added to the generator will be `export`ed as a type.

```rust
generator.generate(
	output_dir.join("types.ts"),
	schema::typescript::TypeScriptRenderer::default(),
);
```

## Features

The following Cargo features are available:

- `config` (default) - Enables configuration support (all the above stuff).
- `url` (default) - Enables loading, extending, and parsing configs from URLs.

### Parsing

- `json` (default) - Enables JSON.
- `toml` - Enables TOML.
- `yaml` - Enables YAML.

### Validation

- `valid_email` - Enables email validation with the `schematic::validate::email` function.
- `valid_url` - Enables URL validation with the `schematic::validate::url` and `url_secure`
  functions.

### Schema generation

- `schema` - Generates schemas for schematic types and built-in Rust types.
- `json_schema` - Enables JSON schema generation.
- `typescript` - Enables TypeScript types generation.

- `type_chrono` - Implements schematic for the `chrono` crate.
- `type_regex` - Implements schematic for the `regex` crate.
- `type_relative_path` - Implements schematic for the `relative-path` crate.
- `type_rust_decimal` - Implements schematic for the `rust_decimal` crate.
- `type_semver` - Implements schematic for the `semver` crate.
- `type_serde_json` - Implements schematic for the `serde_json` crate.
- `type_serde_toml` - Implements schematic for the `toml` crate.
- `type_serde_yaml` - Implements schematic for the `serde_yaml` crate.
- `type_url` - Implements schematic for the `url` crate.
- `type_version_spec` - Implements schematic for the `version_spec` crate.
- `type_warpgate` - Implements schematic for the `warpgate` crate.
