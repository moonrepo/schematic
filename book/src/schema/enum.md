# Enums

The [`EnumType`][enum] can be used to represent a list of [literal values](./literal.md).

```rust
use schematic::{Schematic, Schema, SchemaBuilder, schema::{EnumType, LiteralValue}};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.enumerable(EnumType {
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
visibility, you can define the `variants` field and pass a map of
[`SchemaField`](https://docs.rs/schematic/latest/schematic/struct.SchemaField.html)s.

```rust
schema.enumerable(EnumType {
	values: vec![
		LiteralValue::String("debug".into()),
		LiteralValue::String("error".into()),
		LiteralValue::String("warning".into()),
	],
	variants: Some(IndexMap::from_iter([
		(
			"Debug".into(),
			Box::new(SchemaField {
				comment: Some("Shows debug messages and above".into()),
				schema: Schema::literal_value(LiteralValue::String("debug".into())),
				..SchemaField::default()
			})
		),
		(
			"Error".into(),
			Box::new(SchemaField {
				comment: Some("Shows only error messages".into()),
				schema: Schema::literal_value(LiteralValue::String("error".into())),
				..SchemaField::default()
			})
		),
		(
			"Warning".into(),
			Box::new(SchemaField {
				comment: Some("Shows warning and error messages".into()),
				schema: Schema::literal_value(LiteralValue::String("warning".into())),
				..SchemaField::default()
			})
		),
	])),
	..EnumType::default()
})
```

> This comes in handy when working with specific generators, like TypeScript.

> `variants` is the source of truth, and `values` is a derived subset of it. A variant whose schema
> holds no literal value contributes no entry to `values`, so the two can differ in length.

## Default value

The `default_index` field points at the entry that is the default. It indexes `variants` when that
map is set, and `values` otherwise, so prefer the
[`EnumType::get_default()`](https://docs.rs/schematic/latest/schematic/schema/struct.EnumType.html#method.get_default)
and
[`EnumType::set_default()`](https://docs.rs/schematic/latest/schematic/schema/struct.EnumType.html#method.set_default)
methods over indexing either list yourself.

```rust
let mut ty = EnumType::new([
	LiteralValue::String("debug".into()),
	LiteralValue::String("error".into()),
]);

// Returns false and keeps the current default when no entry matches
ty.set_default(LiteralValue::String("error".into()));

assert_eq!(ty.get_default(), Some(&LiteralValue::String("error".into())));
```

[enum]: https://docs.rs/schematic/latest/schematic/schema/struct.EnumType.html
