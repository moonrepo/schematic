# Schematic

Schematic is a library that provides:

- A layered serde-driven configuration system with support for merge strategies, validation rules,
  environment variables, and more!
- A schema modeling system that can be used to generate TypeScript types, JSON schemas, and more!

Both of these features can be used independently or together.

```
cargo add schematic
```

Get started: https://moonrepo.github.io/schematic

## Configuration

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

Define a struct or enum and derive the `Config` trait.

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

## Schemas

Define a struct or enum and derive or implement the `Schematic` trait.

```rust
use schematic::Schematic;

#[derive(Schematic)]
struct Task {
	command: String,
	args: Vec<String>,
	env: HashMap<String, String>,
}
```

Then generate output in multiple formats, like JSON schemas or TypeScript types, using the schema
type information.

```rust
use schematic::schema::{SchemaGenerator, TypeScriptRenderer};

let mut generator = SchemaGenerator::default();
generator.add::<Task>();
generator.generate(output_dir.join("types.ts"), TypeScriptRenderer::default())?;
```
