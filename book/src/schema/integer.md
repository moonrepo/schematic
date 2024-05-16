# Integers

The [`IntegerType`][integer] can be used to represent an integer (number).

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{IntegerType, IntegerKind}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.integer(IntegerType {
			kind: IntegerKind::U32,
			..IntegerType::default()
		})
	}
}
```

If you're only defining the `kind` field, you can use the shorthand
[`IntegerType::new_kind()`](https://docs.rs/schematic/latest/schematic/struct.IntegerType.html#method.new_kind)
method.

```rust
schema.integer(IntegerType::new_kind(IntegerKind::U32));
```

> Automatically implemented for `usize`-`u128` and `isize`-`i128`.

## Default value

To customize the default value for use within [generators](./generator/index.md), pass the desired
value to the [`IntegerType`][integer] constructor.

```rust
schema.integer(IntegerType::new(IntegerKind::I32, 100));
// Or
schema.integer(IntegerType::new_unsigned(IntegerKind::U32, 100));
```

## Settings

The following fields can be passed to [`IntegerType`][integer], which are then fed into the
[generator](./generator/index.md).

### Enumerable

The `enum_values` field can be used to specify a list of literal values that are allowed for the
field.

```rust
IntegerType {
	// ...
	enum_values: Some(vec![0, 25, 50, 75, 100]),
}
```

### Formats

The `format` field can be used to associate semantic meaning to the integer, and how the integer
will be used and displayed.

```rust
IntegerType {
	// ...
	format: Some("age".into()),
}
```

> This is primarily used by JSON Schema.

### Min/max

The `min` and `max` fields can be used to specify the minimum and maximum inclusive values allowed.
Both fields accept a non-zero number, and can be used together or individually.

```rust
IntegerType {
	// ...
	min: Some(0), // >0
	max: Some(100), // <100
}
```

These fields are not exclusive and do not include the lower and upper bound values. To include them,
use `min_exclusive` and `max_exclusive` instead.

```rust
IntegerType {
	// ...
	min_exclusive: Some(0), // >=0
	max_exclusive: Some(100), // <=100
}
```

### Multiple of

The `multiple_of` field can be used to specify a value that the integer must be a multiple of.

```rust
IntegerType {
	// ...
	multiple_of: Some(25), // 0, 25, 50, etc
}
```

[integer]: https://docs.rs/schematic/latest/schematic/schema/struct.IntegerType.html
