# Structs

The [`StructType`][struct] can be used to represent a struct with explicitly named fields and typed
values. This is also known as a "shape" or "model".

```rust
use schematic::{Schematic, Schema, SchemaBuilder, SchemaType, schema::StructType};

impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		schema.structure(StructType {
			fields: IndexMap::from_iter([
				(
					"name".into(),
					Box::new(SchemaField {
						comment: Some("Name of the user".into()),
						schema: schema.infer::<String>(),
						..SchemaField::default()
					})
				),
				(
					"age".into(),
					Box::new(SchemaField {
						comment: Some("Age of the user".into()),
						schema: schema.nest().integer(IntegerType::new_kind(IntegerKind::U16)),
						..SchemaField::default()
					})
				),
				(
					"active".into(),
					Box::new(SchemaField {
						comment: Some("Is the user active".into()),
						schema: schema.infer::<bool>(),
						..SchemaField::default()
					})
				),
			]),
			..StructType::default()
		})
	}
}
```

If you're only defining `fields`, you can use the shorthand
[`StructType::new()`](https://docs.rs/schematic/latest/schematic/struct.StructType.html#method.new)
method. When using this approach, the `Box`s are automatically inserted for you.

```rust
schema.structure(StructType::new([
	(
		"name".into(),
		SchemaField {
			comment: Some("Name of the user".into()),
			schema: schema.infer::<String>(),
			..SchemaField::default()
		}
	),
	// ...
]));
```

> Automatically implemented for `Duration`, `SystemTime`, `Range`, and `RangeInclusive`. These look
> like scalars but serde encodes them as maps, so the schema describes the map.

Fields keep the order they were inserted in, which is the declaration order when the schema comes
from a derive. [Generators](./generator/index.md) sort them by name when rendering, so generated
output stays alphabetical regardless of how the source type is written.

## Optional and nullable fields

Each field's `SchemaField` describes two separate things about its value:

- `optional` - The key may be left out of the input entirely.
- `nullable` - The key may be present with a `null` value.

Renderers read them independently. JSON Schema leaves an optional field out of `required`, and gives
a nullable one an `anyOf` that includes `null`. TypeScript marks an optional property with `?`, and
a nullable one with `| null`.

When the schema comes from a derive, they follow what serde accepts:

| Field                                       | Optional | Nullable | TypeScript              |
| ------------------------------------------- | -------- | -------- | ----------------------- |
| `name: String`                              | No       | No       | `name: string`          |
| `#[setting(default = "x")] name: String`    | Yes      | No       | `name?: string`         |
| `name: Option<String>`                      | Yes      | Yes      | `name?: string \| null` |
| `#[setting(required)] name: Option<String>` | No       | No       | `name: string`          |

An `Option` is optional as well as nullable, as serde fills in an omitted one with `None`. A
`#[serde(default)]` on the field, or on the struct, makes a field optional too. A required `Option`
must be given a value, which `null` isn't, so it's described by the type it holds.

> For a [config](../config/index.md), a field that isn't optional describes what a source should
> provide, which is stricter than what loads. Every setting of a [partial](../config/partial.md)
> may be omitted, and one that no source provides falls back to its default value. Only
> `#[setting(required)]` makes a missing value an error when loading, as described in
> [required settings](../config/struct/validate.md#required-settings).

## Settings

The following fields can be passed to [`StructType`][struct], which are then fed into the
[generator](./generator/index.md).

### Required fields

The `required` field can be used to specify a list of fields that are required for the struct.

```rust
StructType {
	// ...
	required: Some(vec!["name".into()]),
}
```

> This is primarily used by JSON Schema.

### Unknown fields

The `allow_unknown_fields` field marks a struct that accepts keys other than its fields, as serde
does unless `#[serde(deny_unknown_fields)]`. It defaults to `false`, closing the struct, which JSON
Schema renders as `additionalProperties: false`.

```rust
StructType {
	// ...
	allow_unknown_fields: true,
}
```

> The derive sets it to what's deserialized: a config's partial denies unknown fields unless
> `#[config(allow_unknown_fields)]`, while a type that only derives `Schematic` follows its own
> `#[serde(deny_unknown_fields)]`.

[struct]: https://docs.rs/schematic/latest/schematic/schema/struct.StructType.html
