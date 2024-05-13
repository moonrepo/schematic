# Unknown

The [`SchemaType::Unknown`][unknown] variant can be used to represent an unknown type. This is
sometimes known as an "any" or "mixed" type.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType};

impl Schematic for T {
	fn build_schema(schema: SchemaBuilder) -> Schema {
		schema.build()
	}
}
```

The [`SchemaType::Unknown`][unknown] variant is also the default variant, and the default
implementation for
[`Schematic::build_schema()`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html#method.build_schema),
so the above can simply be written as:

```rust
impl Schematic for T {}
```

[unknown]: https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Unknown
