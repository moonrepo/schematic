# Literals

The [`LiteralType`](https://docs.rs/schematic/latest/schematic/schema/struct.LiteralType.html)
paired with
[`SchemaType::Literal`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Literal)
can be used to represent a literal primitive value, such as a string or number.

```rust
use schematic::{Schematic, SchemaType, schema::{LiteralType, LiteralValue}};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::literal(LiteralValue::String("enabled".into()))
		// Or, etc
		SchemaType::literal(LiteralValue::Uint(100))
	}
}
```

> The [`LiteralValue`](https://docs.rs/schematic/latest/schematic/schema/enum.LiteralValue.html)
> type is used by other schema types for their default or enumerable values.
