# Developer experience

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**X1. Attribute typos print a raw `Debug` dump of the darling error** · fixed on `0.21-devx`

`Container::new` returns a `darling::Result` that collects every attribute that fails to parse, on
the container, a field, a variant, or a variant's value, and the macro shell calls
`write_errors()`. So `#[setting(defualt = 1)]` is a compile error at the attribute: ``Unknown field:
`defualt`. Did you mean `default`?``. If the container's own attributes fail, its settings aren't
analyzed, as their checks depend on them. `Container::from` stays, panicking with the errors'
message, for the snapshot tests. A misused attribute that parses still panics.

**X2. Derive panics name neither field nor variant** · fixed on `0.21-devx`

A panic about a setting or variant is prefixed with it, like ``Setting `port`: `` (``Setting
`File.path`: `` within a named variant, ``Setting `1`: `` by position) or ``Variant `A`: ``, through
`Field::fail`, `Value::fail`, and `Variant::fail`. `Value` holds the description as `owner`, so a
panic while generating code names the setting too. Rust names, not serde names. Spanned errors
through the X1 accumulator were set aside, as checks made while generating code can't return one.

`#[setting(default)]` writes any expression as is, only converting a string and sorting a bare path
into a value or a handler.

**X3. TOML syntax errors are a bare `Handler` error** · fixed on `0.21-devx`

A syntax error is a `ParserError` from `error.span()`, so it gets the file, the `config::parse::failed`
code, and `set_help`. Its path is `.`, the root, as serde_path_to_error writes it for the other
formats, as TOML parses the whole document first. YAML's merge-key error and RON's constructor
error, which also had an empty path, use `.` too.

**X4. YAML type and unknown-field errors, and all RON data errors, have no span** · fixed on
`0.21-devx`

- YAML: on failure, the content is deserialized directly, only to locate the error, and its span is
  used when it fails at the same path, so an error a merge key causes isn't labeled elsewhere.
- RON: `de.span_error(err)`, with the whitespace RON's span starts with skipped, so the label lands
  on the token. It also calls `de.end()`, so trailing characters fail, as with `ron::from_str`.

**X5. Validation errors are blamed on the last file** · fixed on `0.21-devx`

The final pass names every source and `environment variables`, when they set something, like
`Failed to validate a.json, b.json, environment variables.`, or the config's name with neither. The
defaults are never listed. A single source reads as before. Exact blame, the layer that set the
failing path, isn't built.

**X6. Errors say `<unknown>` with default features** · fixed on `0.21-devx`

The loader's name falls back to `short_type_name::<T>()` (`Wrapper<String>`, each path shortened),
and a code source is located by its file name.

**X7. Runtime handler errors don't name the setting** · fixed on `0.21-devx`

`InvalidDefaultValue { path, error }` and a new `TransformFailed { path, error }`, with the path
passed by the generated code: the serde name, `Variant.field` for a named variant's field, or the
variant for a tuple variant, `Variant[i]` with several values. Only the setting's own name, not its
parent's. A transform may return any `Display` error.

**X8. Env parse errors drop the reason** · fixed on `0.21-devx`

`Failed to parse "70000" into u16: number too large to fit in target type.`, which needs
`<T as FromStr>::Err: Display`. `parse_bool` fails on an unknown value, treats an empty one as
unset, and accepts `0`, `false`, `no`, `off`, `disable`, and `disabled`. The env docs point at it
for a `bool`.

**X9. Partial type names leak into user errors** · won't fix

serde_derive writes "expected struct PartialChild" from the Rust type's name, so `rename` doesn't
change it, only `#[serde(expecting)]` does. That would also replace each enum variant's message,
and a struct's "with N elements" in a length error. The maintainer chose to leave the `Partial`
names, untagged errors included.

**X10. Default features include no format** · fixed on `0.21-devx`

`NoMatchingFormat`'s help names the feature that adds a format for the extension, when it's
disabled. The README and book say formats are opt-in, and the README that `serde` is a dependency.
No format was added to the defaults.

**X11. Small papercuts** · fixed on `0.21-devx`, but one

- `ConfigLoadResult` implements `Debug`, by hand, as a derive can't bound each layer's partial.
- `ConfigError` locations stay plain `String`s, by decision. moon builds `ConfigError::Validator`
  itself, and since X5 a location can be a list.
- An optional missing file isn't a layer.
- The map key of a nested collection joins a validation path through a call spanned to the
  setting's type, so a key without `Display` fails to compile there.
