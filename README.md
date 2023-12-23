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
