# Unions

The [`UnionType`][union] paired with
[`SchemaType::Union`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Union)
can be used to represent a list of heterogeneous schema types (variants), in which a value must
match one or more of the types.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{UnionType, UnionOperator}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.union(UnionType {
			operator: UnionOperator::AnyOf,
			variants_types: vec![
				Box::new(schema.infer::<String>()),
				Box::new(schema.infer::<bool>()),
				Box::new(schema.nest().integer(IntegerType::new_kind(IntegerKind::U32))),
			],
			..UnionType::default()
		});
		schema.build()
	}
}
```

If you're only defining the `variants_types` field, you can use the shorthand
[`UnionType::new_any()`](https://docs.rs/schematic/latest/schematic/struct.UnionType.html#method.new_any)
(any of) or
[`UnionType::new_one()`](https://docs.rs/schematic/latest/schematic/struct.UnionType.html#method.new_one)
(one of) methods. When using this approach, the `Box`s are automatically inserted for you.

```rust
// Any of
schema.union(UnionType::new_any([
	schema.infer::<String>(),
	schema.infer::<bool>(),
	schema.nest().integer(IntegerType::new_kind(IntegerKind::U32)),
]));

// One of
schema.union(UnionType::new_one([
	// ...
]));
```

## Operators

Unions support 2 kinds of operators, any of and one of, both of which can be defined with the
`operator` field.

- Any of requires the value to match any of the variants.
- One of requires the value to match _only one_ of the variants.

```rust
UnionType {
	// ...
	operator: UnionOperator::OneOf,
}
```

[union]: https://docs.rs/schematic/latest/schematic/schema/struct.UnionType.html
