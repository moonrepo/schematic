# Objects

The [`ObjectType`][object] can be used to represent a key-value object of homogenous types. This is
also known as a map, record, keyed object, or indexed object.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::ObjectType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.object(ObjectType {
			key_type: Box::new(schema.infer::<String>()),
			value_type: Box::new(schema.infer::<String>()),
			..ObjectType::default()
		});
		schema.build()
	}
}
```

If you're only defining the `key_type` and `value_type` fields, you can use the shorthand
[`ObjectType::new()`](https://docs.rs/schematic/latest/schematic/struct.ObjectType.html#method.new)
method.

```rust
schema.object(ObjectType::new(schema.infer::<String>(), schema.infer::<String>()));
```

> Automatically implemented for `BTreeMap` and `HashMap`.

## Settings

The following fields can be passed to [`ObjectType`][object], which are then fed into the
[generator](./generator/index.md).

### Length

The `min_length` and `max_length` fields can be used to restrict the length (key-value pairs) of the
object. Both fields accept a non-zero number, and can be used together or individually.

```rust
ObjectType {
	// ...
	min_length: Some(1),
	max_length: Some(10),
}
```

### Required keys

The `required` field can be used to specify a list of keys that are required for the object, and
must exist when the object is validated.

```rust
ObjectType {
	// ...
	required: Some(vec!["foo".into(), "bar".into()]),
}
```

> This is primarily used by JSON Schema.

[object]: https://docs.rs/schematic/latest/schematic/schema/struct.ObjectType.html
