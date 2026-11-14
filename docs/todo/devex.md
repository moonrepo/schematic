# Developer experience

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**X1. Attribute typos print a raw `Debug` dump of the darling error** ✅ · med-high

- **Scenario:** `#[setting(defualt = 1)]` panics with `Result::unwrap()` on
  `Error { kind: UnknownField(... did_you_mean: Some((0.96, "default")) ...) }`, pointing at
  `#[derive(Config)]`. Darling already has the span and the suggestion.
- **Where:** `crates/core/src/container.rs:54-55`, `field.rs:86-87`, `variant.rs:81-82`.
- **Fix:** have `Container::from` return `darling::Result<Self>`, accumulate errors, and call
  `write_errors()` in the macro shell. The intentional panics elsewhere are untouched.

**X2. Derive panics name neither field nor variant** ✅ · med

- **Context:** about 45 `panic!`s, such as "Cannot use `required` with `default`…", don't say which
  field.
- **Defaults dump syn:** `default = PathBuf::from("a").join("b")` is rejected with roughly 1 KB of
  `Expr::MethodCall { … }` (`value.rs:212`).
- **Fix:** prefix each message with the setting name, or route through `syn::Error::new_spanned`
  with the X1 accumulator. Accept any other `Expr` verbatim as a default; only `Lit::Str` and bare
  paths need special handling.

**X3. TOML syntax errors are a bare `Handler` error** ✅ · med

- **Effect:** there is no file name, no `config::parse::failed` code, and `set_help` is not applied.
- **Where:** `formats/toml.rs:22-24`.
- **Fix:** build a `ParserError` from `error.span()`, as the second pass already does. About 5
  lines.

**X4. YAML type and unknown-field errors, and all RON data errors, have no span** ✅ · med

- **YAML:** it goes through `Value` first for merge keys, so its most common error ("invalid type:
  string, expected u16") gets no snippet. YAML is moon's format.
- **Fix:**
  - YAML: on failure, re-run a direct deserialize only to locate the error.
  - RON: `de.span_error(err)`.

**X5. Validation errors are blamed on the last file** ✅ · med

- **Scenario:** an env var failing `in_range`, or a `required` miss, reports "Failed to validate
  …/b.json", even when `b.json` doesn't contain the setting.
- **Where:** `loader.rs:144-146`.
- **Fix:** use a neutral location for the final pass, or list all sources. Exact blame needs G4.

**X6. Errors say `<unknown>` with default features** ✅ · med

- **Cause:** the loader name comes from `schema_name()`, which is `None` without the `schema`
  feature. Tests pass only because dev-deps enable `schema`.
- **Where:** `loader.rs:48`, `:236`.
- **Fix:** fall back to the last segment of `type_name::<T>()`, and use `Source::Code.path` as the
  location for code sources.

**X7. Runtime handler errors don't name the setting** ✅ · low-med

- **Scenario:** a failing transform yields just `nope`, and a failing default yields `Invalid default
  value. bad default`.
- **Fix:** have codegen wrap the error with the setting key.

**X8. Env parse errors drop the reason** ✅ · med

- **Messages:** `PORT=""` and `PORT=70000` both give "Failed to parse into the correct type".
- **`parse_bool`:**
  - `parse_bool("ture")` silently returns `false`, and an empty value is also `false`.
  - A plain `bool` rejects `FLAG=1`, and the env docs don't point at `parse_bool`.
- **Where:** `internal/mod.rs:267-275`, `env.rs:17-22`.
- **Fix:**
  - Include `type_name::<T>()` and the `FromStr` error in the message.
  - Make `parse_bool` error on unknown values, with an empty value meaning `None`.

**X9. Partial type names leak into user errors** ✅ · low

- **Scenario:** messages read "expected struct PartialChild" and "any variant of PartialMode".
- **Fix:** emit `#[serde(rename = "<name>")]` on the partial when none is given, and pass
  `get_name()` to `untagged_error_message`.

**X10. Default features include no format** 📖 · med

- **Scenario:** `cargo add schematic` plus the README's `.code("secure: false", "config.yml")`
  returns `NoMatchingFormat` at runtime, and `.url()` doesn't compile without the `url` feature.
- **Fix:** do one of the following, in order of cost:
  - Make `NoMatchingFormat`'s help name the feature to enable (cheapest).
  - Fix the README and book.
  - Add `yaml` or `json` to the defaults.

**X11. Small papercuts** · low

- `ConfigLoadResult` has no `Debug`, so `load().unwrap_err()` doesn't compile ✅.
- `ConfigError` locations are plain `String`s, not `Source` 📖.
- An optional missing file still appears in `layers` as an empty layer 📖.
- A non-`Display` map key on a nested collection fails inside generated validate code, with the
  error pointing at the derive ✅.
