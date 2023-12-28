# Arrays

The [`ArrayType`][array] paired with
[`SchemaType::Array`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Array)
can be used to represent a variable list of homogeneous values of a given type, as defined by
`items_type`. For example, a list of strings:

```rust
use schematic::{Schematic, SchemaType, schema::ArrayType};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Array(ArrayType {
			items_type: Box::new(SchemaType::string()),
			..ArrayType::default()
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

> Automatically implemented for `Vec`, `BTreeSet`, `HashSet`, `[T; N]`, and `&[T]`.

## Settings

The following fields can be passed to [`ArrayType`][array], which are then fed into the
[generator](./generator/index.md).

### Contains

The `contains` field can be enabled to indicate that the array must contain at least one item of the
type defined by `items_type`, instead of all items.

```rust
ArrayType {
	// ...
	contains: Some(true),
}
```

### Length

The `min_length` and `max_length` fields can be used to restrict the length of the array. Both
fields accept a non-zero number, and can be used together or individually.

```rust
ArrayType {
	// ...
	min_length: Some(1),
	max_length: Some(10),
}
```

### Uniqueness

The `unique` field can be used to indicate that all items in the array must be unique. Note that
Schematic _does not_ verify uniqueness.

```rust
ArrayType {
	// ...
	unique: Some(true),
}
```

[array]: https://docs.rs/schematic/latest/schematic/schema/struct.ArrayType.html
