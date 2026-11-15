# Settings

Settings are the individual fields of a [`Config` struct](./struct/index.md), the variants of a
[`Config` enum](./struct/index.md), or the fields of a
[named variant](./struct/index.md#enum-variants), and can be annotated with the optional `#[setting]`
attribute.

## Third-party types

A setting that isn't wrapped in `Option` falls back to `Default::default()` when the config provides
no value and no [default value](./struct/default.md) is declared, so every such setting has to
implement `Default`, even one marked [`required`](./struct/validate.md#required-settings). Plenty of
third-party types don't, and being foreign, can't be given it.

For the two we support directly, we provide a newtype that does:

| Setting                                                                              | Wraps            | Default | Cargo feature  |
| ------------------------------------------------------------------------------------ | ---------------- | ------- | -------------- |
| [`RegexSetting`](https://docs.rs/schematic/latest/schematic/struct.RegexSetting.html) | `regex::Regex`   | `.`     | `type_regex`   |
| [`VersionSetting`](https://docs.rs/schematic/latest/schematic/struct.VersionSetting.html) | `semver::Version` | `0.0.0` | `type_semver` |

```rust
use schematic::{Config, RegexSetting, VersionSetting};

#[derive(Config)]
struct AppConfig {
	pub allowed: RegexSetting,
	pub version: VersionSetting,
}
```

Each derefs to the type it wraps, so its methods can be called directly, and each parses from a
string, so the wrapper works without the upstream crate's `serde` feature.

> `semver::VersionReq` needs no wrapper, as it already defaults to `*`. Types that are only ever
> optional don't need one either, since an `Option` setting defaults to `None`.

### Required traits

Every setting is held by the [partial](./partial.md), which derives `Clone`, `Debug`, `Default`,
`PartialEq`, `Deserialize`, and `Serialize`, so a setting's type has to implement those too, along
with [`Schematic`](../schema/index.md). `Debug` and `PartialEq` aren't needed by the partial itself,
but deriving them lets partials be printed and compared, and lets a struct that holds one derive
them as well. A type that lacks one, like `regex::Regex`, which has no `PartialEq`, fails to compile
at `#[derive(Config)]`, so wrap it in a newtype that implements it, as `RegexSetting` does by
comparing patterns.

## Attribute fields

The following fields are supported for the `#[setting]` field/variant attribute:

- `default` - Sets the [default value](./struct/default.md). On an enum variant, it names the
  variant the partial defaults to, and on a variant holding a value, can also provide that value.
- `env` _(field only)_ - Sets the [environment variable](./struct/env.md) to receive a value from.
  On a field of a named variant, it's read once a source has [picked](./struct/env.md#enums) the
  variant.
- `env_prefix` _(field only)_ - Adds an [environment variable](./struct/env.md#nested-prefixes)
  prefix for the nested config this field holds. Requires `nested`.
- `exclude` - Omits the field or variant from the generated [schema](../schema/index.md). Has no
  effect without the `schema` Cargo feature.
- `extend` _(struct only)_ - Enables a configuration to [extend other configs](./struct/extend.md).
- `merge` - Defines a function to use for [merging values](./struct/merge.md).
- `nested` - Marks the field as using a [nested `Config`](./nested.md).
- `null` _(enum only)_ - Marks the variant as representing `null`, so it is never tagged.
- `parse_env` _(field only)_ - Parses the [environment variable](./struct/env.md) value using a
  function. Requires either `env`, or an `env_prefix` on the container.
- `partial` - Forwards attributes to the field on the [partial](./partial.md). May be repeated.
- `required` - Marks the setting as [required](./struct/validate.md#required-settings), so that
  loading fails when no source provides a value, instead of falling back to a default.
- `transform` - Defines a function to use for [transforming values](./struct/transform.md).
- `validate` - Defines a function to use for [validating values](./struct/validate.md).

And the following for serde compatibility:

- `alias`
- `flatten`
- `rename`
- `skip` - Also left out of the generated [schema](../schema/index.md), as it's never read or
  written, so its type needn't implement `Schematic`.
- `skip_deserializing` - Hidden in the generated schema, as it's written but never read.
- `skip_serializing`
- `skip_serializing_if` - Only honored from `#[setting]`, and written against the partial's
  `Option`. The `#[serde]` form is left for the full type, whose field is not wrapped.
- `rename_all` _(named variant only)_ - Applies a casing to the fields of the variant, over the
  enum's `rename_all_fields`.
- `untagged` _(enum only)_

### Serde support

A handful of serde attribute fields are currently supported (above) and will apply a `#[serde]`
attribute to the [partial](./partial.md) implementation.

```rust
#[derive(Config)]
struct Example {
	#[setting(rename = "type")]
	pub type_of: SomeEnum,
}
```

> These values can also be applied using `#[serde]`, which is useful if you want to apply them to
> the main struct as well, and not just the partial struct. When a field is set through both, the
> `#[setting]` value wins, except for `alias`, where the two lists are merged.

Names are used exactly as written. There is no default casing, so a field named `allowed_hosts` is
parsed as `allowed_hosts` unless a `rename` or `rename_all` says otherwise. An explicit `rename` is
never re-cased by `rename_all`.
