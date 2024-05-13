# Booleans

The [`BooleanType`][boolean] can be used to represent a boolean `true` or `false` value. Values that
evaluate to true or false, such as 1 and 0, are not accepted by the schema.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::BooleanType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.boolean_default();
		schema.build()
	}
}
```

> Automatically implemented for `bool`.

## Default value

To customize the default value for use within [generators](./generator/index.md), pass the desired
value to the [`BooleanType`][boolean] constructor.

```rust
schema.boolean(BooleanType::new(true));
```

[boolean]: https://docs.rs/schematic/latest/schematic/schema/struct.BooleanType.html
