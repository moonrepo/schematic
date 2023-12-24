# Arrays

The [`ArrayType`][array] paired with
[`SchemaType::Array`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Array)
can be used to represent a list of values of a given type. For example, a list of strings:

```rust
use schematic::{Schematic, SchemaType, schema::ArrayType};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Array(ArrayType {
			items_type: Box::new(SchemaType::string()),
			// ...
		})
	}
}
```

If you're only defining the `items_type` field, you can use the shorthand
[`SchemaType::array()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.array)
method.

```rust
SchemaType::array(SchemaType::string());
```

[array]: https://docs.rs/schematic/latest/schematic/schema/struct.ArrayType.html
