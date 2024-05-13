# Strings

The [`StringType`][string] can be used to represent a sequence of bytes, you know, a string.

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{StringType, IntegerKind}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.string_default();
		schema.build()
	}
}
```

> Automatically implemented for `char`, `str`, `String`, `Path`, `PathBuf`, `Ipv4Addr`, `Ipv6Addr`,
> `SystemTime`, and `Duration`.

## Default value

To customize the default value for use within [generators](./generator/index.md), pass the desired
value to the [`StringType`][string] constructor.

```rust
schema.string(StringType::new("abc"));
```

## Settings

The following fields can be passed to [`StringType`][string], which are then fed into the
[generator](./generator/index.md).

### Enumerable

The `enum_values` field can be used to specify a list of literal values that are allowed for the
field.

```rust
StringType {
	// ...
	enum_values: Some(vec!["a".into(), "b".into(), "c".into()]),
}
```

### Formats

The `format` field can be used to associate semantic meaning to the string, and how the string will
be used and displayed.

```rust
StringType {
	// ...
	format: Some("url".into()),
}
```

> This is primarily used by JSON Schema.

### Length

The `min_length` and `max_length` fields can be used to restrict the length of the string. Both
fields accept a non-zero number, and can be used together or individually.

```rust
StringType {
	// ...
	min_length: Some(1),
	max_length: Some(10),
}
```

### Patterns

The `pattern` field can be used to define a regex pattern that the string must abide by.

```rust
StringType {
	// ...
	format: Some("version".into()),
	pattern: Some("\d+\.\d+\.\d+".into()),
}
```

> This is primarily used by JSON Schema.

[string]: https://docs.rs/schematic/latest/schematic/schema/struct.StringType.html
