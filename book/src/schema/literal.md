# Literals

The [`LiteralType`](https://docs.rs/schematic/latest/schematic/schema/struct.LiteralType.html) can
be used to represent a literal primitive value, such as a string or number.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{LiteralType, LiteralValue}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.literal(LiteralType::new(LiteralValue::String("enabled".into())))
		// Or
		schema.literal_value(LiteralValue::String("enabled".into()))
	}
}
```

> The [`LiteralValue`](https://docs.rs/schematic/latest/schematic/schema/enum.LiteralValue.html)
> type is used by other schema types for their default or enumerable values.
