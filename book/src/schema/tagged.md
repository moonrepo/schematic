# Tagged variants

The [`TaggedType`][tagged] paired with
[`SchemaType::Tagged`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Tagged)
represents an enum variant that serde writes with its name, as a tag around or within the value it
holds. It's a variant of a [union](./union.md), and the derive creates one for every variant of an
enum that's tagged, so it's rarely built by hand.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, schema::{TaggedType, TagFormat, UnionType}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.union(UnionType::new_any([
			Schema::tagged(TaggedType::new(
				TagFormat::Internal { tag: "type".into() },
				"git",
				Some(schema.infer::<GitSource>()),
			)),
			Schema::tagged(TaggedType::new(
				TagFormat::Internal { tag: "type".into() },
				"none",
				None,
			)),
		]))
	}
}
```

## Formats

The `format` field is how serde writes the variant's `name`, from the enum's `#[serde(tag)]` and
`#[serde(content)]` attributes.

- `TagFormat::External` (default) - `{ "name": value }`.
- `TagFormat::Internal { tag }` - `{ "tag": "name", ...value }`.
- `TagFormat::Adjacent { tag, content }` - `{ "tag": "name", "content": value }`.

A unit variant holds no `value`, so it's only its tag, like `{ "tag": "name" }`. An externally
tagged unit variant is written as its name alone, so it's a [literal](./literal.md) instead, and an
untagged variant is its value alone, without a tagged type around it.

The value is kept apart from the tag, rather than within a struct of them, as a named type's schema
is shared by every type that holds it. The tag's `env_var` is the environment variable that picks
the variant, from an enum's `env_prefix`.

## Rendering

A renderer can render each format as it sees fit, through `SchemaRenderer::render_tagged`, which
defaults to rendering the struct serde writes for the variant, from
[`TaggedType::to_schema()`](https://docs.rs/schematic/latest/schematic/schema/struct.TaggedType.html#method.to_schema).
An internally tagged variant holding a named struct is flattened into a struct of its tag, as adding
the tag to the named struct would claim it for every type that holds it, so a TypeScript type
renders it as `{ type: 'git' } & GitSource`.

A renderer that reads the shape of the structs it renders, rather than rendering each tag format,
can replace every tagged variant with that struct up front, with
[`Schema::expand_tagged()`](https://docs.rs/schematic/latest/schematic/schema/struct.Schema.html#method.expand_tagged).

[tagged]: https://docs.rs/schematic/latest/schematic/schema/struct.TaggedType.html
