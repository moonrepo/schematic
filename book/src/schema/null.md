# Nulls

The [`SchemaType::Null`][null] variant can be used to represent a literal `null` value. This works
best when paired with unions or fields that need to be [nullable](#marking-as-nullable).

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.custom(SchemaType::Null);
		schema.build()
	}
}
```

> Automatically implemented for `()` and `Option<T>`.

## Marking as nullable

If you want a concrete schema to also accept null (an `Option`al value), you can use the
[`SchemaBuilder::nullable()`](https://docs.rs/schematic/latest/schematic/struct.SchemaBuilder.html#method.nullable)
method. Under the hood, this will create a union of the defined type, and the null type.

```rust
// string | null
schema.nullable(schema.infer::<String>());
```

[null]: https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Null
