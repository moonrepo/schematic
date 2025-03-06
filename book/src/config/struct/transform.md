# Transforming values

Sometimes a value is configured by a user, but it needs to be transformed in some way to be usable,
for example, expanding file system paths to absolute from relative.

This can be achieved with the `#[setting(transform)]` attribute field, which requires a reference to
a function to call. Only values with a defined value are transformed, while optional values remain
`None`.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(transform = make_absolute)]
	pub env_file: Option<PathBuf>,
}
```

> Transformations happen during the finalize phase, _after_ [environment variables](./env.md) are
> inherited, and _before_ it is [validated](./validate.md).

## Transform handler function

When defining a custom `transform` function, the defined value and [context](../context.md) are
passed as arguments, and the function must return the transformed result.

Here's an example of the transform function above.

```rust
fn make_absolute(value: PathBuf, context: &Context) -> TransformResult<PathBuf> {
	Ok(if value.is_absolute() {
		value
	} else {
		context.root.join(value)
	})
}
```

### Nested values

Transformers can also be used on [nested configs](../nested.md), but when defining the transformer
function, the value being transformed is the _[partial nested config](../partial.md)_, not the final
one. For example:

```rust
fn transform_nested(value: PartialChildConfig, context: &Context) -> TransformResult<PartialChildConfig> {
	Ok(value)
}

#[derive(Config)]
struct ParentConfig {
	#[setting(nested, transform = transform_nested)]
	pub child: ChildConfig,
}
```

### Context handling

If you're not using [context](../context.md), you can use `()` as the context type, or rely on
generic inferrence.

```rust
fn using_unit_type<T>(value: T, _: &()) -> TransformResult<T> {
	// ...
}

fn using_generics<T, C>(value: T, _: &C) -> TransformResult<T> {
	// ...
}
```
