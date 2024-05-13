# Types

Schema types are the building blocks when modeling your schema. They are used to define the explicit
shape of your types, data, or configuration. This type information is then passed to a
[generator](./generator/index.md), which can then generate and render the schema types in a variety
of formats.

- [Arrays](./array.md)
- [Booleans](./boolean.md)
- [Enums](./enum.md)
- [Floats](./float.md)
- [Integers](./integer.md)
- [Literals](./literal.md)
- [Nulls](./null.md)
- [Objects](./object.md)
- [Strings](./string.md)
- [Structs](./struct.md)
- [Tuples](./tuple.md)
- [Unions](./union.md)
- [Unknown](./unknown.md)

## Defining names

Schemas can be named, which is useful for referencing them in other types when generating code. By
default the [`Schematic`][schematic] derive macro will use the name of the type, but when
implementing the trait manually, you can use the
[`Schematic::schema_name()`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html#method.schema_name)
method.

```rust
impl Schematic for T {
	fn schema_name() -> Option<String> {
		Some("CustomName".into())
	}
}
```

> This method is optional, but is encouraged for non-primitive types. It will associate references
> between types, and avoid circular references.

## Inferring schemas

When building a schema, you'll almost always need to reference schemas from other types that
implement [`Schematic`][schematic]. To do so, you can use the
[`SchemaBuilder.infer::<T>()`](https://docs.rs/schematic/latest/schematic/struct.SchemaBuilder.html#method.build_schema)
method, which will create a nested builder, and build an isolated schema based on its
implementation.

```rust
struct OtherType {}

impl Schematic for OtherType {
	// ...
}


impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		let builtin_type = schema.infer::<String>();
		let custom_type = schema.infer::<OtherType>();

		// ...
	}
}
```

## Creating nested schemas

When building a schema, you may have situations where you need to build nested schemas, for example,
within struct fields. You _cannot_ use the type-based methods on `SchemaBuilder`, as they mutate the
current builder. Instead you must created another builder, which can be achieved with the
[`SchemaBuilder.nest()`](https://docs.rs/schematic/latest/schematic/struct.SchemaBuilder.html#method.nest)
method.

```rust
impl Schematic for T {
	fn build_schema(mut schema: SchemaBuilder) -> Schema {
		// Mutates self
		schema.string_default();

		// Creates a new builder and mutates it
		schema.nest().string_default();

		// ...
	}
}
```

[schematic]: https://docs.rs/schematic/latest/schematic/trait.Schematic.html
