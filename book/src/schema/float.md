# Floats

The [`FloatType`][float] can be used to represent a float or double.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{FloatType, FloatKind}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.float(FloatType {
			kind: FloatKind::F32,
			..FloatType::default()
		});
		schema.build()
	}
}
```

If you're only defining the `kind` field, you can use the shorthand
[`FloatType::new_kind()`](https://docs.rs/schematic/latest/schematic/struct.FloatType.html#method.new_kind)
method.

```rust
schema.float(FloatType::new_kind(FloatKind::F32));
```

> Automatically implemented for `f32` and `f64`.

## Default value

To customize the default value for use within [generators](./generator/index.md), pass the desired
value to the [`FloatType`][float] constructor.

```rust
schema.float(FloatType::new_32(32.0));
// Or
schema.float(FloatType::new_64(64.0));
```

## Settings

The following fields can be passed to [`FloatType`][float], which are then fed into the
[generator](./generator/index.md).

### Enumerable

The `enum_values` field can be used to specify a list of literal values that are allowed for the
field.

```rust
FloatType {
	// ...
	enum_values: Some(vec![0.0, 0.25, 0.5, 0.75, 1.0]),
}
```

### Formats

The `format` field can be used to associate semantic meaning to the float, and how the float will be
used and displayed.

```rust
FloatType {
	// ...
	format: Some("currency".into()),
}
```

> This is primarily used by JSON Schema.

### Min/max

The `min` and `max` fields can be used to specify the minimum and maximum inclusive values allowed.
Both fields accept a non-zero number, and can be used together or individually.

```rust
FloatType {
	// ...
	min: Some(0.0), // >0
	max: Some(1.0), // <1
}
```

These fields are not exclusive and do not include the lower and upper bound values. To include them,
use `min_exclusive` and `max_exclusive` instead.

```rust
FloatType {
	// ...
	min_exclusive: Some(0.0), // >=0
	max_exclusive: Some(1.0), // <=1
}
```

### Multiple of

The `multiple_of` field can be used to specify a value that the float must be a multiple of.

```rust
FloatType {
	// ...
	multiple_of: Some(0.25), // 0.0, 0.25, 0.50, etc
}
```

[float]: https://docs.rs/schematic/latest/schematic/schema/struct.FloatType.html
