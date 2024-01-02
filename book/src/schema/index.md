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

Our derive macro will always implement schemas using the default state of [`SchemaType`][schematype]
and [associated types](./types.md). If you want these types to use custom settings, you can
implement the [`Schematic`][schematic] trait and
[`Schematic::generate_schema()`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html#method.generate_schema)
method manually.

```rust
use schematic::{Schematic, SchemaField, SchemaType, schema::*};

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
	fn generate_schema() -> SchemaType {
		SchemaType::structure([
			SchemaField::new("name", SchemaType::String(StringType {
				min_length: Some(1),
				..StringType::default()
			})),
			SchemaField::new("age", SchemaType::integer(IntegerKind::Usize)),
			SchemaField::new("status", SchemaType::infer::<UserStatus>()),
		])
	}
}
```

> Learn more about our [supported types](./types.md).

## Cargo features

The following Cargo features are available:

#### Renderers

Learn more about [renderers](./generator/index.md).

- `json_schema` - Enables JSON schema generation.
- `typescript` - Enables TypeScript types generation.

#### External types

Learn more about [external types](./external.md).

- `type_chrono` - Implements schematic for the `chrono` crate.
- `type_regex` - Implements schematic for the `regex` crate.
- `type_relative_path` - Implements schematic for the `relative-path` crate.
- `type_rust_decimal` - Implements schematic for the `rust_decimal` crate.
- `type_semver` - Implements schematic for the `semver` crate.
- `type_serde_json` - Implements schematic for the `serde_json` crate.
- `type_serde_toml` - Implements schematic for the `toml` crate.
- `type_serde_yaml` - Implements schematic for the `serde_yaml` crate.
- `type_url` - Implements schematic for the `url` crate.

[schematic]: https://docs.rs/schematic/latest/schematic/trait.Schematic.html
[schematype]: https://docs.rs/schematic/latest/schematic/enum.SchemaType.html
