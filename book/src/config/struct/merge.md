# Merge strategies

A common requirement for configuration is to merge multiple sources/layers into a final result. By
default Schematic will replace the previous setting value with the next value if the next value is
`Some`, but sometimes you want far more control, like shallow or deep merging collections.

This can be achieved with the `#[setting(merge)]` attribute field, which requires a reference to a
function to call.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(merge = schematic::merge::append_vec)]
	pub allowed_hosts: Vec<String>,
}

#[derive(Config)]
enum Projects {
	#[setting(merge = schematic::merge::append_vec)]
	List(Vec<String>),
	// ...
}
```

> We provide a handful of built-in merge functions in the
> [`merge` module](https://docs.rs/schematic/latest/schematic/merge/index.html).

## Merge handler function

You can also define your own function for merging values.

When defining a custom `merge` function, the previous value, next value, and
[context](../context.md) are passed as arguments, and the function must return an optional merged
result. If `None` is returned, neither value will be used.

Here's an example of the merge function above.

```rust
fn append_vec<T>(mut prev: Vec<T>, next: Vec<T>, context: &Context) -> MergeResult<Vec<T>>> {
    prev.extend(next);

    Ok(Some(prev))
}

#[derive(Config)]
struct ExampleConfig {
	#[setting(merge = append_vec)]
	pub field: Vec<String>,
}
```

### Context handling

If you're not using [context](../context.md), you can use `()` as the context type, or rely on
generic inferrence.

```rust
fn using_unit_type<T>(prev: T, next: T, _: &()) -> MergeResult<T> {
	// ...
}

fn using_generics<T, C>(prev: T, next: T, _: &C) -> MergeResult<T> {
	// ...
}
```
