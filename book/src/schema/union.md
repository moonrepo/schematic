# Unions

The [`UnionType`][union] paired with
[`SchemaType::Union`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Union)
can be used to represent a list of heterogeneous schema types (variants), in which a value must
match one or more of the types.

```rust
use schematic::{Schematic, SchemaType, schema::{UnionType, UnionOperator}};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Union(UnionType {
			operator: UnionOperator::AnyOf,
			variants_types: vec![
				Box::new(SchemaType::string()),
				Box::new(SchemaType::bool()),
				Box::new(SchemaType::integer(IntegerKind::U32)),
			],
			..UnionType::default()
		})
	}
}
```

If you're only defining the `variants_types` field, you can use the shorthand
[`SchemaType::union()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.union)
(any of) or
[`SchemaType::union_one()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.union_one)
(one of) methods.

```rust
SchemaType::union([
	SchemaType::string(),
	SchemaType::bool(),
	SchemaType::integer(IntegerKind::U32),
]);
// Or
SchemaType::union_one([]);
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
