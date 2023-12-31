# Structs

The [`StructType`][struct] paired with
[`SchemaType::Struct`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Struct)
can be used to represent a struct with explicitly named fields and typed values. This is also known
as a "shape" or "model".

Unlike other schema types, structs are composed of
[`SchemaField`](https://docs.rs/schematic/latest/schematic/struct.SchemaField.html)s.

```rust
use schematic::{Schematic, SchemaField, SchemaType, schema::StructType};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Struct(StructType {
			fields: vec![
				SchemaField {
					name: "name".into(),
					description: Some("Name of the user".into()),
					type_of: SchemaType::string(),
					..SchemaField::default()
				},
				SchemaField {
					name: "age".into(),
					description: Some("Age of the user".into()),
					type_of: SchemaType::integer(IntegerKind::U16),
					..SchemaField::default()
				},
				SchemaField {
					name: "active".into(),
					description: Some("Is the user active".into()),
					type_of: SchemaType::boolean(),
					..SchemaField::default()
				},
			],
			..StructType::default()
		})
	}
}
```

If you're only defining `fields`, you can use the shorthand
[`SchemaType::structure()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.structure)
method.

```rust
SchemaType::structure([
	// ...
]);
```

## Settings

The following fields can be passed to [`StructType`][struct], which are then fed into the
[generator](./generator/index.md).

### Required fields

The `required` field can be used to specify a list of fields that are required for the struct.

```rust
StructType {
	// ...
	required: Some(vec!["name".into()]),
}
```

> This is primarily used by JSON Schema.

[struct]: https://docs.rs/schematic/latest/schematic/schema/struct.StructType.html
