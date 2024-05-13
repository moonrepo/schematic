# Enums

The [`EnumType`][enum] can be used to represent a list of [literal values](./literal.md).

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::{EnumType, LiteralValue}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.enumerable(EnumType {
			values: vec![
				LiteralValue::String("debug".into()),
				LiteralValue::String("error".into()),
				LiteralValue::String("warning".into()),
			],
			..EnumType::default()
		});
		schema.build()
	}
}
```

If you're only defining the `values` field, you can use the shorthand
[`EnumType::new()`](https://docs.rs/schematic/latest/schematic/struct.EnumType.html#method.new)
method.

```rust
schema.enumerable(EnumType::new([
	LiteralValue::String("debug".into()),
	LiteralValue::String("error".into()),
	LiteralValue::String("warning".into()),
]));
```

## Detailed variants

If you'd like to provide more detailed information for each variant (value), like descriptions and
visibility, you can define the `variants` field and pass a list of
[`Schema`](https://docs.rs/schematic/latest/schematic/struct.Schema.html)s.

```rust
use schematic::Schema;

schema.enumerable(EnumType {
	values: vec![
		LiteralValue::String("debug".into()),
		LiteralValue::String("error".into()),
		LiteralValue::String("warning".into()),
	],
	variants: Some(vec![
		Schema {
			name: "debug".into(),
			description: Some("Shows debug messages and above".into()),
			ty: SchemaType::literal(LiteralValue::String("debug".into())),
			..Schema::default()
		},
		Schema {
			name: "error".into(),
			description: Some("Shows only error messages".into()),
			ty: SchemaType::literal(LiteralValue::String("error".into())),
			..Schema::default()
		},
		Schema {
			name: "warning".into(),
			description: Some("Shows warning and error messages".into()),
			ty: SchemaType::literal(LiteralValue::String("warning".into())),
			..Schema::default()
		},
	]),
	..EnumType::default()
})
```

> This comes in handy when working with specific generators, like TypeScript.

[enum]: https://docs.rs/schematic/latest/schematic/schema/struct.EnumType.html
