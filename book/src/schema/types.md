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

## Names

Schema types can be named, which is useful for referencing them in other types when generating code.
By default the [`Schematic`](https://docs.rs/schematic/latest/schematic/trait.Schematic.html) derive
macro will use the name of the type, but when implementing the trait manually, you can use
[`SchemaType::set_name()`](https://docs.rs/schematic/latest/schematic/enum.SchemaType.html#method.set_name).

```rust
let mut schema = SchemaType::string();
schema.set_name("CustomName");
```
