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

> This type also contains all fields from the upstream
> [`SchemaSettings`](https://docs.rs/schemars/latest/schemars/gen/struct.SchemaSettings.html) type
> from the `schemars` crate. Refer to their documentation for more information.

### Markdown descriptions

By default, the `description` field in the JSON schema specification is supposed to be a plain text
string, but some tools support markdown through another field called `markdownDescription`.

To support this pattern, enable the `markdown_description` option, which will inject the
`markdownDescription` field if markdown was detected in the `description` field.

```rust
JsonSchemaOptions {
	// ...
	markdown_description: true,
}
```

> This is a non-standard extension to the JSON schema specification.

### Required fields

When a struct is rendered, automatically mark all non-`Option` struct fields as required, and
include them in the JSON schema
[`required` field](https://json-schema.org/understanding-json-schema/reference/object#required).
This is enabled by default.

```rust
JsonSchemaOptions {
	// ...
	mark_struct_fields_required: false,
}
```

### Field titles

The JSON schema specification supports a
[`title` annotation](https://json-schema.org/understanding-json-schema/reference/annotations) for
each field, which is a human-readable string. By default this is the name of the Rust struct, enum,
or type field.

But depending on the tool that consumes the schema, this may not be the best representation. As an
alternative, the `set_field_name_as_title` option can be enabled to use the field name itself as the
`title`.

```rust
JsonSchemaOptions {
	// ...
	set_field_name_as_title: true,
}
```
