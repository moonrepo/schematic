# Settings

Settings are the individual fields/members of a [`Config` struct](../index.md), and can be annotated
with the optional `#[setting]` attribute.

## Attribute fields

The following fields are supported for the `#[setting]` field attribute:

- `default` - Sets the [default value](./default.md).
- `env` - Sets the [environment variable](./env.md) to receive a value from.
- `extend` - Enables a configuration to [extend other configs](./extend.md).
- `merge` - Defines a function to use for [merging values](./merge.md).
- `nested` - Marks the field as using a [nested `Config`](../nested.md).
- `parse_env` - Parses the [environment variable](./env.md) value using a function.
- `validate` - Defines a function to use for [validating values](./validate.md).

And the following for serde compatibility:

- `flatten`
- `rename`
- `skip`
- `skip_deserializing`
- `skip_serializing`

### Serde support

A handful of serde attribute fields are currently supported (above) and will apply a `#[serde]`
attribute to the [partial](../partial.md) setting.

```rust
#[derive(Config)]
struct Example {
	#[setting(rename = "type")]
	type_of: SomeEnum,
}
```

> These values can also be applied using `#[serde]`, which is useful if you want to apply them to
> the main struct as well, and not just the partial struct.
