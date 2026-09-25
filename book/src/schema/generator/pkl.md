# Pkl modules

> Requires the `renderer_pkl_schema` Cargo feature.

With our
[`PklSchemaRenderer`](https://docs.rs/schematic/latest/schematic/schema/pkl_schema/struct.PklSchemaRenderer.html),
you can generate [Pkl modules](https://pkl-lang.org/main/current/language-reference/index.html#modules)
for all types that implement
[`Schematic`](https://docs.rs/schematic/latest/schematic/schema/trait.Schematic.html). Each struct
and enum becomes its own module, with its Rust types converted to Pkl types, so that a Pkl config
can amend it and be type checked by Pkl itself.

To utilize, instantiate a generator, add types to render, and generate a module for every type into
a directory. Modules import each other by relative path, so they must stay together.

```rust
use schematic::schema::{SchemaGenerator, PklSchemaRenderer};

let mut generator = SchemaGenerator::default();
generator.add::<ServerConfig>();

PklSchemaRenderer::default().generate_all(&generator, output_dir.join("pkl"))?;
```

Each module is named after its type with a `.pkl` extension, so the above writes `ServerConfig.pkl`,
and a module for every type it nests. A config then amends the module of its root type.

```pkl
amends "pkl/ServerConfig.pkl"

port = 3000
timeout = 30.s
tags { "web"; "api" }
```

Like a [JSON schema](./json-schema.md), rendering through `SchemaGenerator::generate()` produces a
single document, which is the module of the _last type to be added to `SchemaGenerator`_. The
modules it imports are not written, so prefer `generate_all()`.

## Module structure

A struct becomes a module that declares each of its fields as a property, with its doc comment and
deprecation carried over. Fields are rendered in alphabetical order, and skipped fields are omitted.

```pkl
/// Configures the HTTP server.
module ServerConfig

import "LogLevel.pkl"
import "TlsConfig.pkl"

/// The level to log at.
level: LogLevel.LogLevel

/// The port to listen on.
port: UInt16 = 8080

/// TLS settings, when serving over HTTPS.
tls: TlsConfig?
```

A module is a type, but it can't be a union or a string, so every other type, such as an enum,
becomes a module that declares a type alias of the same name. It is referenced as `Name.Name`. The
default variant of an enum is marked with `*`, which makes it the default of any property of that
type that isn't nullable.

```pkl
/// The level to log at.
typealias LogLevel = *"info"|"debug"|"error"
```

A struct without a name of its own, such as a variant of a tagged enum, becomes a class in the
module it's found in, named after its position, such as `ActionRun` for the `Run` variant of
`Action`. A class is referenced from other modules as `Module.Class`.

```pkl
typealias Action = "Stop"|ActionRun

class ActionRun {
  Run: String
}
```

## Inheritance

A struct that flattens another struct with `#[serde(flatten)]` includes all of its fields, which is
what extending a module does. Its module extends the module of the flattened struct, which is then
declared `open`, as Pkl requires to extend it.

```pkl
open module BaseConfig

shared: String = "shared"
```

```pkl
module ServerConfig

extends "BaseConfig.pkl"

port: UInt16 = 8080
```

When the struct declares nothing but the flattened field, it has the same shape as the struct it
flattens, so its module amends that module instead. Pkl can't use a module that amends another as a
type, so this only happens when no other type refers to the struct.

A module can only extend one other, so any further flattened structs have their fields declared
directly. A flattened map, which collects unknown keys, cannot be declared at all, as a Pkl object
only accepts the properties its type declares.

## Types

Rust types are converted to their closest Pkl types, and constraints, such as a minimum or a
pattern, become [type constraints](https://pkl-lang.org/main/current/language-reference/index.html#type-constraints).

| Rust | Pkl |
| --- | --- |
| `bool` | `Boolean` |
| `i8`, `i16`, `i32` | `Int8`, `Int16`, `Int32` |
| `i64`, `i128`, `isize` | `Int` |
| `u8`, `u16`, `u32` | `UInt8`, `UInt16`, `UInt32` |
| `u64`, `u128`, `usize` | `UInt` |
| `f32`, `f64` | `Float` |
| `char` | `Char` |
| `String`, `PathBuf`, `Url`, ... | `String` |
| `Vec<T>` | `Listing<T>` |
| `HashSet<T>`, `BTreeSet<T>` | `Listing<T>(isDistinct)` |
| `[T; N]` | `Listing<T>(length == N)` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `Mapping<K, V>` |
| `(A, B)` | `Pair<A, B>` |
| `(A, B, C, ...)` | `Listing<A\|B\|C>(length == N)` |
| `Option<T>` | `T?` |
| `std::time::Duration` | `Duration` |
| `serde_json::Value`, ... | `Any` |
| Unit-only enum | `"a"\|"b"\|"c"` |

`Pair` and `Duration` are decoded by the [Pkl format](../../config/experimental.md) into the same
shapes as a Rust tuple and `Duration`, so `timeout = 30.s` loads as expected. Pkl's own JSON and YAML
renderers can't output them without a converter, however.

Arrays and maps are typed as `Listing` and `Mapping`, which a config can amend, such as
`tags { "web" }`. A value created with `List()` or `Map()` doesn't pass that type check, so use
`new Listing {}` and `new Mapping {}` instead.

A few things can't be expressed in Pkl, and are rendered as close as possible:

- A setting named `output` can't be declared by a module, as every module reserves it. It is left
  out, with a comment in its place.
- Pkl rejects a type alias that refers to itself, so a recursive enum refers to itself as `Any`. A
  recursive struct refers to its own module, which is fine.
- A tuple of more than two values can only be typed as a list of each of its types, so the position
  of each type isn't checked.

## Options

Custom options can be passed to the renderer using
[`PklSchemaOptions`](https://docs.rs/schematic/latest/schematic/schema/pkl_schema/struct.PklSchemaOptions.html).

```rust
use schematic::schema::PklSchemaOptions;

PklSchemaRenderer::new(PklSchemaOptions {
	// ...
	..PklSchemaOptions::default()
});
```

### Indentation

The indentation of class properties can be customized using the `indent_char` option. By default
this is 2 spaces.

```rust
PklSchemaOptions {
	// ...
	indent_char: "\t".into(),
}
```

### Required fields

By default, struct fields are rendered as they are declared. A field that isn't optional has no
default, so a config must set it, and a field with a default value renders it as the property's
default.

Pkl outputs every property of a module, including the defaults a config didn't set. A config is only
one [layer](../../config/index.md) of many however, so a default it outputs overrides the value of
any layer beneath it, such as a config it [extends](../../config/struct/extend.md). Disable the
`mark_struct_fields_required` option to render every property of a struct's module as nullable,
without a default, so that a config only outputs the settings it sets, and everything else falls
back to the defaults of the Rust types.

```rust
PklSchemaOptions {
	// ...
	mark_struct_fields_required: false,
}
```

```pkl
// Enabled
port: UInt16 = 8080
host: String

// Disabled
port: UInt16?
host: String?
```

Classes, such as the variants of an enum, are always rendered as declared, as their value is only
valid in full. Rendering [partial types](../../config/partial.md) has the same effect, as every
field of a partial is optional, but names every module after its partial type.
