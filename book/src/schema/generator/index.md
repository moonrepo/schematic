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

fn main() {
	let mut generator = SchemaGenerator::default();
}
```

From here, for every type that implements
[`Schematic`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html) and you want to
include in the generated output, call
[`SchemaGenerator::add()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.add).
If you only have a [`SchemaType`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html),
you can use the
[`SchemaGenerator::add_schema()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.add_schema)
method instead.

```rust
use schematic::schema::SchemaGenerator;

fn main() {
	let mut generator = SchemaGenerator::default();
	generator.add::<FirstConfig>();
	generator.add::<SecondConfig>();
	generator.add::<ThirdConfig>();
}
```

> We'll recursively add referenced and nested schemas for types that are added. No need to
> explicitly add all required types!

From here, call
[`SchemaGenerator::generate()`](https://docs.rs/schematic/latest/schematic/schema/struct.SchemaGenerator.html#method.generate)
to render the schemes with a chosen [renderer](#renderers) to an output file of your choice. This
method can be called multiple times, each with a different output file or renderer.

```rust
use schematic::schema::SchemaGenerator;

fn main() {
	let mut generator = SchemaGenerator::default();
	generator.add::<FirstConfig>();
	generator.add::<SecondConfig>();
	generator.add::<ThirdConfig>();
	generator.generate(PathBuf::from("output/file"), CustomRenderer::default())?;
	generator.generate(PathBuf::from("output/another/file"), AnotherRenderer::default())?;
}
```

## Renderers

The following built-in renderers are available, but custom renderers can be created as well by
implementing the
[`SchemaRenderer`](https://docs.rs/schematic/latest/schematic/schema/trait.SchemaRenderer.html)
trait.

- [JSON schemas](./json-schema.md)
- [TypeScript types](./typescript.md)
