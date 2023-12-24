# Settings

Settings are the individual fields of a [`Config` struct](./struct/index.md) or variants of a
[`Config` enum](./struct/index.md), and can be annotated with the optional `#[setting]` attribute.

## Attribute fields

The following fields are supported for the `#[setting]` field/variant attribute:

- `default` - Sets the [default value](./struct/default.md).
- `env` _(struct only)_ - Sets the [environment variable](./struct/env.md) to receive a value from.
- `extend` _(struct only)_ - Enables a configuration to [extend other configs](./struct/extend.md).
- `merge` - Defines a function to use for [merging values](./struct/merge.md).
- `nested` - Marks the field as using a [nested `Config`](./nested.md).
- `parse_env` _(struct only)_ - Parses the [environment variable](./struct/env.md) value using a
  function.
- `validate` - Defines a function to use for [validating values](./struct/validate.md).

And the following for serde compatibility:

- `flatten`
- `rename`
- `skip`
- `skip_deserializing`
- `skip_serializing`

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
> the main struct as well, and not just the partial struct.
