# Composability

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**C1. Nested configs are referenced by bare identifier** · fixed on `0.21-bugs` and `0.21-compose`

Nested configs are called through `<T as Config>` by the path they're written as, so a config from
another module and `Self` work (`0.21-bugs`). Generic nested configs, like `Wrapper<String>`, and a
type parameter that is a config work too (`0.21-compose`), as the layer walk stops at the first type
that isn't a layer. A type parameter still needs the partial's derive bounds, as std's derives
bound every parameter; see `book/src/config/generics.md`.

**C2. One layer walker instead of three** · fixed on `0.21-compose`

`wrap_layer` no longer names collections (`0.21-bugs`), and `impl_full_from_partial_value` and
`impl_partial_finalize_nested` now share `map_layers`, so a layer kind is handled in one place. The
generated code is unchanged; every core snapshot is identical. Validation still walks on its own,
as it borrows rather than rebuilds.

**C3. A library config can't be nested in an app config with a different `Context`** · fixed on
`0.21-compose`

`#[setting(nested, context = func)]` takes a `fn(&Parent) -> &Child` that maps the parent's context
for every call that passes one to the nested config (defaults, finalize, merge, validate).
`schematic::context::unit` covers a child with `Context = ()`. Code without a `context` is
unchanged; the managers gained `_with` variants that take the child's context.

**C4. A hand-written `PartialConfig` is silently skipped when nested** ✅ · med

- **Scenario:** a hand-written impl overrides `validate`, but the parent calls the
  `#[doc(hidden)] validate_with_path`, whose default is `Ok(())`. "level too high" errors at the top
  level and loads `Ok` when nested.
  - The same happens for env: a parent `env_prefix` calls the hidden `env_values_with_prefix`.
- **Where:** `internal/validate.rs:101-105`, `configs.rs:88-95`.
- **Fix:** have `nested_at` call `validate()` and re-path its errors. Alternatively, document the
  hidden methods as the real override points. Small.

**C5. The partial always derives `PartialEq` and `Debug`** ✅ · low-med

- `PartialConfig` doesn't require either, but a setting type without them (a `ConfigEnum` without
  `PartialEq`, `regex::Regex`) fails at the derive. `partial(derive(..))` can only add derives.
- Fix: document the requirement, or drop the two derives (breaking for code comparing partials).

**C6. Multi-value variants can't mix nested and plain positions** ✅ · low

- `#[setting(nested)] Both(Child, bool)` panics on `bool`.
- Fix: read `#[setting(nested)]` from each tuple field's own attributes.

**C7. `ConfigLoader` is neither `Send` nor `Sync`** ✅ · low-med

- The trait objects lack `+ Send + Sync`, so a loader can't live in shared state or a reload task.
- Fix: add the bounds. Mildly breaking for non-`Send` custom formats or cachers.

**C8. Custom renderers can't reuse built-in helpers** 📖 · low

- `schema::renderers` is private. `TemplateContext`, the `template::render_*` helpers,
  `split_doc_tags` and Pkl quoting are unreachable.
- `SchemaRenderer` has 15 required methods and no default impls.
- `SchemaGenerator::generate` only writes files.
- `JsonSchemaRenderer` is `SchemaRenderer<schemars::Schema>`, so a helper generic over
  `SchemaRenderer<String>` can't take it.
- With `schema_serde`, several fields lack `serde(default)`, so hand-authored schemas must spell
  them out.

**C9. `#[setting(skip)]` fields still require `T: Schematic`** ✅ · med

- They are hidden, but still `infer`red and registered (exported as unused TS types and
  definitions).
- Where: `container.rs:946`.
- Fix: treat `skip` like `exclude` in schema codegen.

**C10. Dependencies pulled in without `config`** ✅ · low

- A schema-only build pulls miette, serde-content, thiserror and tracing.
- `env`, `extends`, `validate`, `url`, `json` and `type_regex` without `config` compile deps that do
  nothing.
- `validate` pulls garde with `regex` even for users who only write custom validators.
- Fix: gate these behind `config`.
