# JSON schemas

> Requires the `json_schema` Cargo feature.

With our
[`JsonSchemaRenderer`](https://docs.rs/schematic/latest/schematic/schema/json_schema/struct.JsonSchemaRenderer.html),
you can generate a [JSON Schema](https://json-schema.org/) document for all types that implement
[`Schematic`](https://docs.rs/schematic/latest/schematic/schema/trait.Schematic.html). Internally
this renderer uses the [`schemars`](https://crates.io/crates/schemars) crate to generate the JSON
document.

To utilize, instantiate a generator, add types to render, and generate the output file.

```rust
use schematic::schema::{SchemaGenerator, JsonSchemaRenderer};

let mut generator = SchemaGenerator::default();
generator.add::<CustomType>();
generator.generate(output_dir.join("schema.json"), JsonSchemaRenderer::default())?;
```

> For a reference implementation, check out
> [moonrepo/moon](https://github.com/moonrepo/moon/blob/master/nextgen/config/src/main.rs).

## Root document

Unlike other renderers, a JSON schema represents a single document, with referenced types being
organized into definitions. In Schematic, the _last type to be added to `SchemaGenerator`_ will be
the root document, while all other types will become definitions. For example:

```rust
// These are definitions
generator.add::<FirstConfig>();
generator.add::<SecondConfig>();
generator.add::<ThirdConfig>();

// This is the root document
generator.add::<LastType>();
generator.generate(output_dir.join("schema.json"), JsonSchemaRenderer::default())?;
```

When rendered, will look something like the following:

```json
{
	"$schema": "http://json-schema.org/draft-07/schema#",
	"title": "LastType",
	"type": "object",
	"properties": {
		// Fields in LastType...
	},
	"definitions": {
		// Other types...
	}
}
```

## Options

Custom options can be passed to the renderer using
[`JsonSchemaOptions`](https://docs.rs/schematic/latest/schematic/schema/json_schema/type.JsonSchemaOptions.html).

```rust
use schematic::schema::JsonSchemaOptions;

JsonSchemaRenderer::new(JsonSchemaOptions {
	// ...
	..JsonSchemaOptions::default()
});
```

> This type is just a re-export of the
> [`SchemaSettings`](https://docs.rs/schemars/latest/schemars/gen/struct.SchemaSettings.html) type
> from `schemars`. Refer to their documentation for more information.
