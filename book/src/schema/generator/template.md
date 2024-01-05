# File templates (experimental)

> Requires the `template` and desired [format](../../config/index.md#supported-source-formats) Cargo
> feature.

With our [template renderers](#support-formats), you can generate a file template in a specific
format. This template will include all fields, default values, comments, metadata, and is useful for
situations like configuration templates and scaffolding defaults.

To utilize, instantiate a generator, add types to render, and generate the output file.

```rust
use schematic::Format;
use schematic::schema::{SchemaGenerator, template::*};

let mut generator = SchemaGenerator::default();
generator.add::<CustomType>();
generator.generate(output_dir.join("config.json"), renderer)?;
```

## Support formats

### JSON

The
[`JsonTemplateRenderer`](https://docs.rs/schematic/latest/schematic/schema/json_template/struct.JsonTemplateRenderer.html)
will render JSON templates _without_ comments. Any commented related options will be force disabled.

```rust
use schematic::schema::{JsonTemplateRenderer, TemplateOptions};

JsonTemplateRenderer::default();
JsonTemplateRenderer::new(TemplateOptions::default());
```

### JSONC

The
[`JsoncTemplateRenderer`](https://docs.rs/schematic/latest/schematic/schema/jsonc_template/struct.JsoncTemplateRenderer.html)
will render JSON templates _with_ comments. We suggest using the `.jsonc` file extension.

```rust
use schematic::schema::{JsoncTemplateRenderer, TemplateOptions};

JsoncTemplateRenderer::default();
JsoncTemplateRenderer::new(TemplateOptions::default());
```

### TOML

The
[`TomlTemplateRenderer`](https://docs.rs/schematic/latest/schematic/schema/toml_template/struct.TomlTemplateRenderer.html)
will render TOML templates.

```rust
use schematic::schema::{TomlTemplateRenderer, TemplateOptions};

TomlTemplateRenderer::default();
TomlTemplateRenderer::new(TemplateOptions::default());
```

### YAML

The
[`YamlTemplateRenderer`](https://docs.rs/schematic/latest/schematic/schema/yaml_template/struct.YamlTemplateRenderer.html)
will render YAML templates.

```rust
use schematic::schema::{YamlTemplateRenderer, TemplateOptions};

YamlTemplateRenderer::default();
YamlTemplateRenderer::new(TemplateOptions::default());
```

## Root document

A template represents a single document, typically for a struct. In Schematic, the _last type to be
added to `SchemaGenerator`_ will be the root document, while all other types will be ignored. For
example:

```rust
// These are only used for type information
generator.add::<FirstConfig>();
generator.add::<SecondConfig>();
generator.add::<ThirdConfig>();

// This is the root document
generator.add::<LastType>();
generator.generate(output_dir.join("config.json"), renderer)?;
```

## Caveats

At this time, [arrays](../array.md) and [objects](../object.md) do not support default values, and
will render `[]` and `{}` respectively.

Furthermore, [enums](../enum.md) and [unions](../union.md) only support default values when
explicitly marked as such. For example, with `#[default]`.

And lastly, when we're unsure of what to render for a value, we'll render `null`. This isn't a valid
value for TOML, and may not be what you expect.

## Example output

Given the following type:

```rust
#[derive(Config)]
struct ServerConfig {
	/// The base URL to serve from.
	#[setting(default = "/")]
	pub base_url: String,

	/// The default port to listen on.
	#[setting(default = 8080, env = "PORT")]
	pub port: usize,
}
```

Would render the following formats:

<table>
<tr>
<td>JSON</td>
<td>TOML</td>
</tr>
<tr>
<td>

```json
{
	// The base URL to serve from.
	"base_url": "/",

	// The default port to listen on.
	// @envvar PORT
	"port": 8080
}
```

</td>
<td>

```toml
# The base URL to serve from.
base_url = "/"

# The default port to listen on.
# @envvar PORT
port = 8080
```

</td>
</tr>
</table>

<br />

<table>
<tr>
<td>YAML</td>
</tr>
<tr>
<td>

```yaml
# The base URL to serve from.
base_url: "/"

# The default port to listen on.
# @envvar PORT
port: 8080
```

</td>
</tr>
</table>

> Applying the desired casing for field names should be done with `rename_all` on the container.

## Options

Custom options can be passed to the renderer using
[`TemplateOptions`](https://docs.rs/schematic/latest/schematic/schema/template/struct.TemplateOptions.html).

```rust
use schematic::schema::TemplateOptions;

JsoncTemplateRenderer::new(TemplateOptions {
	// ...
	..TemplateOptions::default()
});
```

> The `format` option is required!

### Indentation

The indentation of the generated template can be customized using the `indent_char` option. By
default this is 2 spaces (` `).

```rust
TemplateOptions {
	// ...
	indent_char: "\t".into(),
}
```

The spacing between fields can also be toggled with the `newline_between_fields` option. By default
this is enabled, which adds a newline between each field.

```rust
TemplateOptions {
	// ...
	newline_between_fields: false,
}
```

### Comments

All Rust doc comments (`///`) are rendered as comments above each field in the template. This can be
disabled with the `comments` option.

```rust
TemplateOptions {
	// ...
	comments: false,
}
```

### Header and footer

The `header` and `footer` options can be customized to add additional content to the top and bottom
of the rendered template respectively.

```rust
TemplateOptions {
	// ...
	header: "$schema: \"https://example.com/schema.json\"\n\n".into(),
	footer: "\n\n# Learn more: https://example.com".into(),
}
```

### Field display

By default all non-skipped fields in the root document (struct) are rendered in the template. If
you'd like to hide certain fields from being rendered, you can use the `hide_fields` option. This
option accepts a list of field names and also supports dot-notation for nested fields.

```rust
TemplateOptions {
	// ...
	hide_fields: vec!["key".into(), "nested.key".into()],
}
```

Additionally, if you'd like to render a field but have it commented out by default, use the
`comment_fields` option instead. This also supports dot-notation for nested fields.

```rust
TemplateOptions {
	// ...
	comment_fields: vec!["key".into(), "nested.key".into()],
}
```

> Field names use the serde cased name, not the Rust struct field name.
