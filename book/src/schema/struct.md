# Structs

The [`StructType`][struct] can be used to represent a struct with explicitly named fields and typed
values. This is also known as a "shape" or "model".

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::StructType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.structure(StructType {
			fields: HashMap::from_iter([
				(
					"name".into(),
					Box::new(SchemaField {
						comment: Some("Name of the user".into()),
						schema: schema.infer::<String>(),
						..SchemaField::default()
					})
				),
				(
					"age".into(),
					Box::new(SchemaField {
						comment: Some("Age of the user".into()),
						schema: schema.nest().integer(IntegerType::new_kind(IntegerKind::U16)),
						..SchemaField::default()
					})
				),
				(
					"active".into(),
					Box::new(SchemaField {
						comment: Some("Is the user active".into()),
						schema: schema.infer::<bool>(),
						..SchemaField::default()
					})
				),
			]),
			..StructType::default()
		})
	}
}
```

If you're only defining `fields`, you can use the shorthand
[`StructType::new()`](https://docs.rs/schematic/latest/schematic/struct.StructType.html#method.new)
method. When using this approach, the `Box`s are automatically inserted for you.

```rust
schema.structure(StructType::new([
	(
		"name".into(),
		SchemaField {
			comment: Some("Name of the user".into()),
			schema: schema.infer::<String>(),
			..SchemaField::default()
		}
	),
	// ...
]));
```

> Automatically implemented for `Duration`, `SystemTime`, `Range`, and `RangeInclusive`. These look
> like scalars but serde encodes them as maps, so the schema describes the map.

Fields keep the order they were inserted in, which is the declaration order when the schema comes
from a derive. [Generators](./generator/index.md) sort them by name when rendering, so generated
output stays alphabetical regardless of how the source type is written.

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
