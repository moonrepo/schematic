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

## Built-in functions

| Function                                            | Merges                                                         |
| --------------------------------------------------- | -------------------------------------------------------------- |
| `discard`                                           | Neither value, so the setting is `None`.                       |
| `preserve`                                          | The previous value.                                            |
| `replace`                                           | The next value, which is the default for every setting.        |
| `append_vec`, `prepend_vec`                         | A `Vec`, with the next items after or before the previous.     |
| `append_vec_unique`                                 | A `Vec`, appending only the next items it doesn't contain yet. |
| `merge_btreemap`, `merge_hashmap`, `merge_indexmap` | A map, the next value of a key replacing the previous.         |
| `merge_btreeset`, `merge_hashset`, `merge_indexset` | A set.                                                         |
| `merge_iter`                                        | Any collection that implements `Extend`, like a `VecDeque`.    |
| `merge_partials`                                    | A map of [nested configs](../nested.md), by key.               |
| `merge_list`                                        | A [`MergeList`](#merging-chosen-by-the-source), as it directs. |

The hash-based functions support any hasher, like `FxHashMap`'s, and the `indexmap` functions
require the `type_indexmap` Cargo feature.

`append_vec_unique` is useful for a list with a default, as it doesn't repeat the default items that
a source lists again.

### Nested maps

A collection of nested configs is replaced by default, as there's no way to pair up its items. For a
map, `merge_partials` pairs them up by key: a config under a key in both layers is merged with its
own strategies, and a key only in the next layer is added.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(nested, merge = schematic::merge::merge_partials)]
	pub plugins: HashMap<String, PluginConfig>,
}
```

It supports `HashMap`, `BTreeMap`, and `IndexMap`, or any map that implements
[`MergeableMap`](https://docs.rs/schematic/latest/schematic/merge/trait.MergeableMap.html). The
configs receive the parent's [context](../context.md), so it can't be combined with a nested
`context` function.

## Merging chosen by the source

A `merge` function decides how every source merges a setting. To let each source decide for itself,
use a [`MergeList`](https://docs.rs/schematic/latest/schematic/struct.MergeList.html). It's merged
with the `merge_list` function without a `merge` of its own.

```rust
use schematic::{Config, MergeList};

#[derive(Config)]
struct AppConfig {
	#[setting(default = MergeList::new(vec!["localhost".into()]))]
	pub allowed_hosts: MergeList<String>,
}
```

A source then writes a list, which replaces the previous items, or a map of how to merge with them,
with any of the `prepend`, `append`, and `replace` keys.

```yaml
# Replaces the previous items, including the default
allowed_hosts: [example.com]

# Adds to the previous items
allowed_hosts:
  prepend: [example.com]
  append: [example.org]
```

A `MergeList` derefs to a slice of its items, so it reads like a `Vec`, and the length validators
accept it. Its schema is a union of the list and the map, so editors suggest both.

The derive knows a `MergeList` by name, including within an `Option` or a variant, so a `merge` of
the setting's own still takes precedence, and a collection of them is replaced like any other.

> Through a type alias the derive can't tell it's a `MergeList`, so add
> `#[setting(merge = schematic::merge::merge_list)]`, or it's replaced like any other setting, and a
> source that appends loses the previous items. Items are plain values for now, not nested configs.

## Merge handler function

You can also define your own function for merging values.

When defining a custom `merge` function, the previous value, next value, and
[context](../context.md) are passed as arguments, and the function must return an optional merged
result. If `None` is returned, neither value will be used.

Here's an example of the merge function above.

```rust
fn append_vec<T>(mut prev: Vec<T>, next: Vec<T>, context: &Context) -> MergeResult<Vec<T>> {
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
