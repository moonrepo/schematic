# Enums

The [`EnumType`][enum] paired with
[`SchemaType::Enum`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#variant.Enum)
can be used to represent a list of [literal values](./literal.md).

```rust
use schematic::{Schematic, SchemaType, schema::{EnumType, LiteralValue}};

impl Schematic for T {
	fn generate_schema() -> SchemaType {
		SchemaType::Enum(EnumType {
			values: vec![
				LiteralValue::String("debug".into()),
				LiteralValue::String("error".into()),
				LiteralValue::String("warning".into()),
			],
			..EnumType::default()
		})
	}
}
```

If you're only defining the `values` field, you can use the shorthand
[`SchemaType::enumerable()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.enumerable)
method.

```rust
SchemaType::enumerable([
	LiteralValue::String("debug".into()),
	LiteralValue::String("error".into()),
	LiteralValue::String("warning".into()),
]);
```

[enum]: https://docs.rs/schematic/latest/schematic/schema/struct.EnumType.html

## Detailed variants

If you'd like to provide more detailed information for each variant (value), like descriptions and
visibility, you can define the `variants` field and pass a list of
[`SchemaField`](https://docs.rs/schematic/latest/schematic/struct.SchemaField.html)s.

```rust
use schematic::SchemaField;

SchemaType::Enum(EnumType {
	values: vec![
		LiteralValue::String("debug".into()),
		LiteralValue::String("error".into()),
		LiteralValue::String("warning".into()),
	],
	variants: vec![
		SchemaField {
			name: "debug".into(),
			description: Some("Shows debug messages and above".into()),
			type_of: SchemaType::literal(LiteralValue::String("debug".into())),
			..SchemaField::default()
		},
		SchemaField {
			name: "error".into(),
			description: Some("Shows only error messages".into()),
			type_of: SchemaType::literal(LiteralValue::String("error".into())),
			..SchemaField::default()
		},
		SchemaField {
			name: "warning".into(),
			description: Some("Shows warning and error messages".into()),
			type_of: SchemaType::literal(LiteralValue::String("warning".into())),
			..SchemaField::default()
		},
	],
	..EnumType::default()
})
```

> This comes in handy when working with specific generators, like TypeScript.
