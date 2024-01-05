# TypeScript types

> Requires the `typescript` Cargo feature.

With our
[`TypeScriptRenderer`](https://docs.rs/schematic/latest/schematic/schema/typescript/struct.TypeScriptRenderer.html),
you can generate [TypeScript types](https://www.typescriptlang.org/) for all types that implement
[`Schematic`](https://docs.rs/schematic/latest/schematic/schema/trait.Schematic.html). To utilize,
instantiate a generator, add types to render, and generate the output file.

```rust
use schematic::schema::{SchemaGenerator, TypeScriptRenderer};

let mut generator = SchemaGenerator::default();
generator.add::<CustomType>();
generator.generate(output_dir.join("types.ts"), TypeScriptRenderer::default())?;
```

> For a reference implementation, check out
> [moonrepo/moon](https://github.com/moonrepo/moon/blob/master/nextgen/config/src/main.rs).

## Options

Custom options can be passed to the renderer using
[`TypeScriptOptions`](https://docs.rs/schematic/latest/schematic/schema/typescript/struct.TypeScriptOptions.html).

```rust
use schematic::schema::TypeScriptOptions;

TypeScriptRenderer::new(TypeScriptOptions {
	// ...
	..TypeScriptOptions::default()
});
```

### Indentation

The indentation of the generated TypeScript code can be customized using the `indent_char` option.
By default this is a tab (`\t`).

```rust
TypeScriptOptions {
	// ...
	indent_char: "  ".into(),
}
```

### Enum types

[Enum types](../enum.md) can be rendered in a format of your choice using the `enum_format` option
and the
[`EnumFormat`](https://docs.rs/schematic/latest/schematic/schema/typescript/enum.EnumFormat.html)
enum. By default enums are rendered as TypeScript string unions, but can be rendered as TypeScript
enums instead.

```rust
TypeScriptOptions {
	// ...
	enum_format: EnumFormat::Enum,
}
```

```ts
// Default
export type LogLevel = "debug" | "info" | "error";

// As enum
export enum LogLevel {
	Debug,
	Info,
	Error,
}
```

Furthermore, the `const_enum` option can be enabled to render `const enum` types instead of `enum`
types. This does not apply when `EnumFormat::Union` is used.

```rust
TypeScriptOptions {
	// ...
	const_enum: true,
}
```

```ts
// Enabled
export const enum LogLevel {}

// Disabled
export enum LogLevel {}
```

### Object types

[Struct types](../struct.md) can be rendered as either TypeScript interfaces or type aliases using
the `object_format` option and the
[`ObjectFormat`](https://docs.rs/schematic/latest/schematic/schema/typescript/enum.ObjectFormat.html)
enum. By default structs are rendered as TypeScript interfaces.

```rust
TypeScriptOptions {
	// ...
	object_format: ObjectFormat::Type,
}
```

```ts
// Default
export interface User {
	name: string;
}

// As alias
export type User = {
	name: string;
};
```

### Type references

In the context of this renderer, a type reference is simply a reference to another type by its name,
and is used by other types of another name. For example, the fields of a struct type may reference
another type by name.

```ts
export type UserStatus = "active" | "inactive";

export interface User {
	status: UserStatus;
}
```

Depending on your use case, this may not be desirable. If so, you can enable the
`disable_references` option, which disables references entirely, and inlines all type information.
So the example above would become:

```rust
TypeScriptOptions {
	// ...
	disable_references: true,
}
```

```ts
export type UserStatus = "active" | "inactive";

export interface User {
	status: "active" | "inactive";
}
```

Additionally, the `exclude_references` option can be used to exclude a type reference by name
entirely from the output, as demonstrated below.

```rust
TypeScriptOptions {
	// ...
	exclude_references: vec!["UserStatus".into()],
}
```

```ts
export interface User {
	status: "active" | "inactive";
}
```

### Importing external types

For better interoperability, you can import external types from other TypeScript modules using the
`external_types` option, which is a map of file paths (relative from the output location) to a list
of types to import from that file. This is useful if:

- You have existing types that aren't generated and want to reference.
- You want to reference types from other generated files, and don't want to duplicate them.

```rust
TypeScriptOptions {
	// ...
	external_types: HashMap::from_iter([
		("./states".into(), vec!["UserStatus".into()]),
	]),
}
```

```ts
import type { UserStatus } from "./states";

export interface User {
	status: UserStatus;
}
```
