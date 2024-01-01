# Generators

The primary benefit of a schema modeling system, is that you can consume this type information to
generate code into multiple output formats. This is a common pattern in many languages, and is a
great way to reduce boilerplate.

In the context of Rust, why use multiple disparate crates, each with their own unique
implementations and `#[derive]` macros, just to generate some output. With Schematic, you can ditch
all of these and use a standardized approach.

## Usage

## Renderers

The following built-in renderers are available, but custom renderers can be created as well by
implementing the
[`SchemaRenderer`](https://docs.rs/schematic/latest/schematic/schema/trait.SchemaRenderer.html)
trait.

- [TypeScript types](./typescript.md)
