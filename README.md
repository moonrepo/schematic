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
