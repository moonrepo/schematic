# Tuples

The [`TupleType`][tuple] paired with
[`SchemaType::Tuple`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Tuple)
can be used to represent a fixed list of heterogeneous values of a given type, as defined by
`items_types`.

```rust
use schematic::{Schematic, SchemaType, schema::{TupleType, IntegerKind}};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Tuple(TupleType {
			items_types: vec![
				Box::new(SchemaType::string()),
				Box::new(SchemaType::bool()),
				Box::new(SchemaType::integer(IntegerKind::U32)),
			],
			..TupleType::default()
		})
	}
}
```

If you're only defining the `items_types` field, you can use the shorthand
[`SchemaType::tuple()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.tuple)
method. When using this approach, the `Box`s are automatically inserted for you.

```rust
SchemaType::tuple([
	SchemaType::string(),
	SchemaType::bool(),
	SchemaType::integer(IntegerKind::U32),
]);
```

> Automatically implemented for tuples of 0-12 length.

[tuple]: https://docs.rs/schematic/latest/schematic/schema/struct.TupleType.html
