# Configuration

> Requires the `config` Cargo feature, which is enabled by default.

The primary feature of Schematic is a layered serde-driven configuration solution, and is powered
through the [`Config`](./struct/index.md) and [`ConfigEnum`](./enum/index.md) traits, and their
associated derive macro. These macros help to generate and automate the following (when applicable):

- Generates a [partial implementation](./partial.md), with all field values wrapped in `Option`.
- Provides [default value](./struct/default.md) and [environment variable](./struct/env.md)
  handling.
- Implements [merging](./struct/merge.md) and [validation](./struct/validate.md) logic.
- Models a [schema](../schema/index.md) (when `schema` Cargo feature enabled).
- And other minor features, like [context & metadata](./context.md#metadata).

The struct or enum that derives `Config` represents the _final state_, after all
[partial layers](./partial.md) have been merged, and default and environment variable values have
been applied. This means that all fields (settings) should _not_ be wrapped in `Option`, unless the
setting is truly optional (think nullable in the config file).

```rust
#[derive(Config)]
struct ExampleConfig {
	pub number: usize,
	pub string: String,
	pub boolean: bool,
	pub array: Vec<String>,
	pub optional: Option<String>,
}
```

> This pattern provides the optimal developer experience, as you can reference the settings as-is,
> without having to unwrap them, or use `match` or `if-let` statements!

## Usage

Define a struct or enum and derive the [`Config`](./struct/index.md) trait. Fields within the struct
(known as [settings](./settings.md)) can be annotated with the `#[setting]` attribute to provide
additional functionality.

```rust
use schematic::Config;

#[derive(Config)]
struct AppConfig {
	#[setting(default = 3000, env = "PORT")]
	pub port: usize,

	#[setting(default = true)]
	pub secure: bool,

	#[setting(default = vec!["localhost".into()])]
	pub allowed_hosts: Vec<String>,
}
```

### Loading sources

When all of your structs and enums have been defined, you can then load, parse, merge, and validate
a configuration from one or many sources. A source is either a file path, secure URL, inline code
string, or a partial in memory.

Begin by importing the
[`ConfigLoader`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html) struct and
initializing it with the [`Config`](https://docs.rs/schematic/latest/schematic/trait.Config.html)
type you want to load.

```rust
use schematic::ConfigLoader;

let loader = ConfigLoader::<AppConfig>::new();
```

From here, you can feed it sources to load. For file paths, use the
[`ConfigLoader::file()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.file)
or
[`ConfigLoader::file_optional()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.file_optional)
methods. For URLs, use the
[`ConfigLoader::url()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.url)
method (requires the `url` Cargo feature, which is _not_ enabled by default). For inline code, use the
[`ConfigLoader::code()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.code)
method, which requires an explicit format, and names the source within errors.

```rust
loader.code("secure: false", "example.yml")?;
loader.file("path/to/config.yml")?;
loader.url("https://ordomain.com/to/config.yaml")?;
```

> The formats to use are derived from the trailing file extension.

An optional file that doesn't exist provides nothing, so it's left out of `ConfigLoadResult.layers`.

A [partial](./partial.md) that's already in memory, like one parsed from command line arguments, is
added with the
[`ConfigLoader::partial()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.partial)
method, along with a name to use within error messages. Nothing is parsed, but it's otherwise loaded
like any other source: merged in the order it was added, validated, and able to
[extend](./struct/extend.md) other sources. A relative file it extends is resolved against the
working directory.

```rust
loader.partial("CLI arguments", PartialAppConfig {
	port: Some(8080),
	..Default::default()
});
```

> Like every source, a partial sits below [environment variables](./struct/env.md), which are
> applied last. For command line arguments to take precedence over the environment, read the
> variables through your argument parser instead, such as with clap's `env` argument, rather than
> with `#[setting(env)]`.

And lastly call the
[`ConfigLoader::load()`](https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html#method.load)
method to generate the final configuration. This methods returns a result, which includes the final
configuration, as well as all of the [partial layers](./partial.md) that were merged into it.

```rust
let result = loader.load()?;

result.config; // AppConfig
result.layers; // Vec<Layer<PartialAppConfig>>
```

The layers are in order of precedence, and each one holds only the values it set, so a value can be
traced to where it came from. The [default values](./struct/default.md) come first, as
`Source::Defaults`, then each source in order, and the
[environment variables](./struct/env.md) last, as `Source::EnvVars`. The default values and
environment variables are left out when they set nothing.

Both are read against what the sources set, so they hold what applies to it: the defaults and
variables of the variant an enum's sources picked, and those of an optional nested config only when
it's present. The environment variables are read first, as a tag's variable can
[pick a variant](./struct/env.md#picking-a-variant), and the defaults then go under the result.

### Automatic schemas

When the `schema` Cargo feature is enabled, the
[`Schematic`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html) trait will be
automatically implemented for all types that implement
[`Config`](https://docs.rs/schematic/latest/schematic/trait.Config.html) and
[`ConfigEnum`](https://docs.rs/schematic/latest/schematic/trait.ConfigEnum.html). You do _not_ and
_should not_ derive both of these together.

```rust
// Correct
#[derive(Config)]
struct AppConfig {}

// Incorrect
#[derive(Config, Schematic)]
struct AppConfig {}
```

## Supported source formats

Schematic is powered entirely by [serde](https://serde.rs), and supports the following formats:

- JSON - Uses `serde_json` and requires the `json` Cargo feature.
- Pkl - Uses `rpkl` and requires the `pkl` Cargo feature.
- RON - Uses `ron` and requires the `ron` Cargo feature.
- TOML - Uses `toml` and requires the `toml` Cargo feature.
- YAML - Uses `serde_norway` and requires the `yaml` Cargo feature.

None of them is enabled by default, so enable the feature of each format you load. A source with no
format to parse it fails to load, with an error that names the feature its extension needs.

## Cargo features

The following Cargo features are available:

- `config` (default) - Enables configuration support (all the above stuff).
- `env` (default) - Enables environment variables for settings.
- `extends` (default) - Enables configs to extend other configs.
- `json` - Enables JSON.
- `pkl` - Enables Pkl.
- `ron` - Enables RON.
- `toml` - Enables TOML.
- `url` - Enables loading, extending, and parsing configs from URLs.
- `validate` (default) - Enables setting value validation.
- `yaml` - Enables YAML.

`env`, `extends`, `url`, and `validate` only apply to configs, so each enables `config` as well. The
dependencies that only configs use are only compiled with `config`, so a build for
[schemas](../schema/index.md) alone, or without any feature, doesn't pull them in.
