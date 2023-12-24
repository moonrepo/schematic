# Fallback variant

Although `ConfigEnum` only supports unit variants, we do support a catch-all variant known as the
"fallback variant", which can be defined with `#[variant(fallback)]`. Fallback variants are
primarily used when parsing from a string, and will be used if no other variant matches.

However, this pattern does have a few caveats:

- Only 1 fallback variant can be defined.
- The fallback variant must be a tuple variant with a single field.
- The field type can be anything and we'll attempt to convert it with `try_into()`.
- The fallback inner value _is not_ casing formatted based on serde's `rename_all`.

```rust
#[derive(ConfigEnum)]
enum Value {
	Foo,
	Bar,
	Baz
	#[variant(fallback)]
	Other(String)
}
```
