# Arrays

The [`ArrayType`][array] can be used to represent a variable list of homogeneous values of a given
type, as defined by `items_type`. For example, a list of strings:

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::ArrayType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.array(ArrayType {
			items_type: Box::new(schema.infer::<String>()),
			..ArrayType::default()
		})
	}
}
```

If you're only defining the `items_type` field, you can use the shorthand
[`ArrayType::new()`](https://docs.rs/schematic/latest/schematic/struct.ArrayType.html#method.new)
method.

```rust
schema.array(ArrayType::new(schema.infer::<String>()));
```

> Automatically implemented for `Vec`, `VecDeque`, `LinkedList`, `BinaryHeap`, `BTreeSet`,
> `HashSet`, `[T; N]`, `[T]`, and `&[T]`.

## Settings

The following fields can be passed to [`ArrayType`][array], which are then fed into the
[generator](./generator/index.md).

### Contains

The `contains` field takes a schema that at least one item must match. It applies alongside
`items_type`, which constrains every item, rather than replacing it.

```rust
ArrayType {
	// ...
	contains: Some(Box::new(Schema::string(StringType::default()))),
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
