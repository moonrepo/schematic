# Generics

[`Config`](./struct/index.md), [`ConfigEnum`](./enum/index.md), and
[`Schematic`](../schema/index.md) can all be derived for types with type parameters. The parameters
are carried into the [partial](./partial.md) and into every generated implementation.

```rust
#[derive(Config)]
struct Wrapper<T> {
	pub inner: T,
	pub label: String,
}
```

## Bounds are not inferred

The derive does not add bounds for you. A partial has to be `Clone + Default + DeserializeOwned +
Schematic + Serialize`, because that is what
[`PartialConfig`](https://docs.rs/schematic/latest/schematic/trait.PartialConfig.html) requires, so
every type parameter needs them too. A type that is missing one fails to compile at the generated
`impl`, not at the definition.

Declaring them once as a trait alias keeps the noise down:

```rust
use serde::{Serialize, de::DeserializeOwned};

pub trait Setting:
	Clone + Default + PartialEq + Serialize + DeserializeOwned + schematic::Schematic
{
}

impl<T> Setting for T where
	T: Clone + Default + PartialEq + Serialize + DeserializeOwned + schematic::Schematic
{
}

#[derive(Config)]
struct Wrapper<T: Setting> {
	pub inner: T,
	pub label: String,
}
```

A generic `Schematic` only needs `T: Schematic`, since it has no partial.

## Serde bounds

The partial is emitted with an explicit `#[serde(bound(deserialize = "T: DeserializeOwned"))]`.
Without it serde would infer a `T: Deserialize<'de>` bound of its own, which is ambiguous against
the `DeserializeOwned` already in scope. Supplying your own bound through
`#[config(partial(serde(bound(...))))]` suppresses the generated one.

## Schema names

Schemas are keyed by name alone, so every instantiation of a generic type would otherwise claim the
same one. The derive appends each type argument to the name, so `Wrapper<String>` becomes
`WrapperString` and its partial `PartialWrapperString`. See
[defining names](../schema/types.md#defining-names) for the manual equivalent.

## Enums

A generic [`ConfigEnum`](./enum/index.md) only makes sense with a
[fallback variant](./enum/fallback.md), since unit variants carry no data. The parameter needs
whatever the generated implementations use: `Default` for
[`variants()`](https://docs.rs/schematic/latest/schematic/trait.ConfigEnum.html#tymethod.variants),
`TryFrom<&str>` for parsing into the fallback, `Display` for formatting back out, and `Schematic`
for the schema name.

```rust
#[derive(Clone, Debug, PartialEq, ConfigEnum)]
enum Value<T>
where
	T: Clone + Default + std::fmt::Display + for<'a> TryFrom<&'a str> + schematic::Schematic,
{
	Known,
	Other,
	#[variant(fallback)]
	Custom(T),
}
```

`Display` is written one arm at a time, so the fallback goes through `T: Display` rather than having
to resolve to a `&str`.
