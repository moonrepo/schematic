# Booleans

The [`BooleanType`][boolean] paired with
[`SchemaType::Boolean`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Boolean)
can be used to represent a boolean `true` or `false` value. Values that evaluate to true or false,
such as 1 and 0, are not accepted by the schema.

```rust
use schematic::{Schematic, SchemaType, schema::BooleanType};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::boolean()
	}
}
```

## Default value

To customize the default value for use within [generators](./generator/index.md), pass the desired
value to the [`BooleanType`][boolean] constructor.

```rust
SchemaType::Boolean(BooleanType::new(true));
```

[boolean]: https://docs.rs/schematic/latest/schematic/schema/struct.BooleanType.html
