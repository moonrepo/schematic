# Code generation

The primary benefit of a schema modeling system, is that you can consume this type information to
generate code into multiple output formats. This is a common pattern in many languages, and is a
great way to reduce boilerplate.

In the context of Rust, why use multiple disparate crates, each with their own unique
implementations and `#[derive]` macros, just to generate some output. With Schematic, you can ditch
all of these and use a single standardized approach.

## Usage

To make use of the generator, import and instantiate our
[`SchemaGenerator`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html).
This is typically done within a one-off `main` function that can be ran from Cargo.

```rust
use schematic::schema::SchemaGenerator;

let mut generator = SchemaGenerator::default();
```

### Adding types

From here, for every type that implements
[`Schematic`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html) and you want to
include in the generated output, call
[`SchemaGenerator::add()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.add).
If you already have a [`Schema`](https://docs.rs/schematic/latest/schematic/struct.Schema.html)
rather than a type, you can use the
[`SchemaGenerator::add_schema()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.add_schema)
method instead.

```rust
use schematic::schema::SchemaGenerator;

let mut generator = SchemaGenerator::default();
generator.add::<FirstConfig>();
generator.add::<SecondConfig>();
generator.add::<ThirdConfig>();
```

> We'll recursively add referenced and nested schemas for types that are added. No need to
> explicitly add all required types!

Schemas are keyed by [name](../types.md#defining-names), so adding two different types that report
the same one panics. Rendering both is impossible, and silently keeping one would leave every
reference to the other pointing at the wrong type. Adding the same type twice is fine.

### Generating output

From here, call
[`SchemaGenerator::generate()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.generate)
to render the schemes with a chosen [renderer](#renderers) to an output file of your choice. This
method can be called multiple times, each with a different output file or renderer.

```rust
use schematic::schema::SchemaGenerator;

let mut generator = SchemaGenerator::default();
generator.add::<FirstConfig>();
generator.add::<SecondConfig>();
generator.add::<ThirdConfig>();
generator.generate(PathBuf::from("output/file"), CustomRenderer::default())?;
generator.generate(PathBuf::from("output/another/file"), AnotherRenderer::default())?;
```

To keep the output in memory instead, such as to compare it, or write it somewhere other than a
file, call
[`SchemaGenerator::render()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.render),
which returns it as a string.

```rust
let output = generator.render(CustomRenderer::default())?;
```

## Renderers

The following built-in renderers are available, but custom renderers can be created as well by
implementing the
[`SchemaRenderer`](https://docs.rs/schematic/latest/schematic/schema/trait.SchemaRenderer.html)
trait.

- [API documentation](./api-docs.md)
- [Config templates](./template.md)
- [JSON schemas](./json-schema.md)
- [Pkl modules](./pkl.md)
- [TypeScript types](./typescript.md)

### Custom renderers

A custom renderer implements `is_reference`, which says whether a named type is rendered by
reference, and `render`, which renders every collected schema. Each `render_*` method, one per type,
defaults to an error naming the renderer and the type, so only the types a renderer supports need
implementing.

```rust
use schematic::schema::{RenderResult, Schema, SchemaRenderer, StringType, StructType};

struct CustomRenderer;

impl SchemaRenderer for CustomRenderer {
	fn is_reference(&self, _name: &str) -> bool {
		false
	}

	fn render_string(&mut self, _string: &StringType, _schema: &Schema) -> RenderResult {
		Ok("string".into())
	}

	fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult {
		// ...
	}

	fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
		// ...
	}
}
```

A renderer whose types render to something other than a string, like the JSON schema renderer, sets
the trait's type parameter, as in `SchemaRenderer<schemars::Schema>`, while `render` always returns
a string. A helper that takes any renderer is generic over both, like
`fn emit<O, R: SchemaRenderer<O>>(renderer: R)`.
