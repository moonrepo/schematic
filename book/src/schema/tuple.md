# Tuples

The [`TupleType`][tuple] can be used to represent a fixed list of heterogeneous values of a given
type, as defined by `items_types`.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{TupleType, IntegerKind}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.tuple(TupleType {
			items_types: vec![
				Box::new(schema.infer::<String>()),
				Box::new(schema.infer::<bool>()),
				Box::new(schema.nest().integer(IntegerType::new_kind(IntegerKind::U32))),
			],
			..TupleType::default()
		});
		schema.build()
	}
}
```

If you're only defining the `items_types` field, you can use the shorthand
[`TupleType::new()`](https://docs.rs/schematic/latest/schematic/struct.TupleType.html#method.new)
method. When using this approach, the `Box`s are automatically inserted for you.

```rust
schema.tuple(TupleType::new([
	schema.infer::<String>(),
	schema.infer::<bool>(),
	schema.nest().integer(IntegerType::new_kind(IntegerKind::U32)),
]));
```

> Automatically implemented for tuples of 0-12 length.

[tuple]: https://docs.rs/schematic/latest/schematic/schema/struct.TupleType.html
