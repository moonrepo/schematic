# Schemas

> Requires the `schema` Cargo feature, which is _not_ enabled by default.

The other feature of Schematic is the ability to model schemas for Rust types using the
[`Schematic`][schematic] trait and associated macro. Schemas are useful for:

- [Generating code](./generator/index.md), documentation, and other formats.
- Ensuring data integrity across systems.
- Standardizing interoperability and enforcing contracts.

## Usage

Define a struct, enum, or type and derive the [`Schematic`][schematic] trait. Our macro will attempt
to convert all fields, variants, values, and generics into a schema representation using
[`SchemaType`][schematype].

```rust
use schematic::Schematic;

#[derive(Schematic)]
enum UserStatus {
	Active,
	Inactive,
}

#[derive(Schematic)]
struct User {
	pub name: String;
	pub age: usize;
	pub status: UserStatus;
}
```

Once a type has a schema associated with it, it can be fed into the
[generator](./generator/index.md).

### Custom implementation

Our derive macro will always implement schemas using the default state of [`Schema`][schema],
[`SchemaType`][schematype], and [associated types](./types.md). If you want these types to use
custom settings, you can implement the [`Schematic`][schematic] trait and
[`Schematic::build_schema()`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html#method.build_schema)
method manually.

The
[`Schematic::schema_name()`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html#method.schema_name)
method is optional, but is encouraged for non-primitive types. It will associate references between
types, and avoid circular references.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::*};

#[derive(Schematic)]
enum UserStatus {
	Active,
	Inactive,
}

struct User {
	pub name: String;
	pub age: usize;
	pub status: UserStatus;
}

impl Schematic for User {
	fn schema_name() -> Option<String> {
		Some("User".into())
	}

	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.structure(StructType::new([
			("name".into(), schema.nest().string(StringType {
				min_length: Some(1),
				..StringType::default()
			})),
			("age".into(), schema.nest().integer(IntegerType::new_kind(IntegerKind::Usize))),
			("status".into(), schema.infer::<UserStatus>()),
		]))
	}
}
```

> Learn more about our [supported types](./types.md).

## Cargo features

The following Cargo features are available:

#### Renderers

Learn more about [renderers](./generator/index.md).

- `renderer_json_schema` - Enables JSON schema generation.
- `renderer_template` - Enables config template generation.
- `renderer_typescript` - Enables TypeScript types generation.

#### External types

Learn more about [external types](./external.md).

- `type_chrono` - Implements schematic for the `chrono` crate.
- `type_indexmap` - Implements schematic for the `indexmap` crate.
- `type_regex` - Implements schematic for the `regex` crate.
- `type_relative_path` - Implements schematic for the `relative-path` crate.
- `type_rust_decimal` - Implements schematic for the `rust_decimal` crate.
- `type_semver` - Implements schematic for the `semver` crate.
- `type_url` - Implements schematic for the `url` crate.

[schematic]: https://docs.rs/schematic/latest/schematic/trait.Schematic.html
[schema]: https://docs.rs/schematic/latest/schematic/struct.Schema.html
[schematype]: https://docs.rs/schematic/latest/schematic/enum.SchemaType.html
