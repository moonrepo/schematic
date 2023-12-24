# Nulls

The [`SchemaType::Null`][null] variant can be used to represent a literal `null` value. This works
best when paired with unions or fields that need to be [nullable](#marking-as-nullable).

```rust
use schematic::{Schematic, SchemaType};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Null
	}
}
```

## Marking as nullable

If you have an existing
[`SchemaType`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html) that you want to
mark as nullable, you can use the
[`SchemaType::nullable()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.nullable)
method, which converts a value to a [union](./union.md) of the original type and
[`SchemaType::Null`][null]. If the type is already a union, it will be appended with
[`SchemaType::Null`][null].

```rust
// string | null
SchemaType::nullable(SchemaType::string());
```

[null]: https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Null
