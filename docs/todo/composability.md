# Composability

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**C1. Nested configs are referenced by bare identifier** ✅ · high

> **Partly fixed on `0.21-bugs`:** nested configs are now called through `<T as Config>` by the
> path they were written as, so `child::Child`, a config from another module, and `Self` work.
> Generic nested configs, like `Wrapper<String>`, still don't, as the layer walk descends into the
> last type argument.

- **Cause:** codegen emits `format_ident!("Partial{}", ident)::default_values` and
  `#ident::from_partial`, while the partial *type* correctly uses `<path as Config>::Partial`.
- **Effects:**
  - `use child::Child;` plus `#[setting(nested)] child: Child` fails with "cannot find type
    `PartialChild`", so users must also import the partial.
  - `child: child::Child` fails.
  - `Option<Box<Self>>` produces 19 errors.
  - Generic nested configs fail. `#[setting(nested)] inner: Wrapper<String>` panics with "must be a
    `Config` type, received `Wrapper<String>`", because the layer walk descends into the last generic
    argument. `inner: T` emits `PartialT`.
- **Where:** `field_value.rs:67`, `:146`, `value.rs:260`, `:716-721`, `:769`, `container.rs:448`.
- **Fix:**
  - Stop the layer walk at the first non-layer segment and keep that whole `syn::Type`.
  - Emit fully qualified calls: `<#ty as schematic::Config>::from_partial(..)` and
    `<<#ty as Config>::Partial as PartialConfig>::default_values(..)`.
  - Substitute `Self`.
- **Cost:** no new impls. This was a limitation of the old derive too, and it is the most common
  composability snag.

**C2. One layer walker instead of three** 📖 · med (maintainability, enables B20 and qualified
paths)

- **Today:** the same `Option`/`Map`/`Set`/`Vec`/`Unknown` match is written out three times: in
  `wrap_layer`, `impl_full_from_partial_value` and `impl_partial_finalize_nested` (`value.rs:22`,
  `:257-330`, `:332-427`).
- **Fix:** one `map_layers(layers, value, f)` helper using `.into_iter().map(..).collect()`. It works
  for any `FromIterator` collection, drops the name heuristics, and stops emitting `Vec::default()`
  and `Arc::new` by bare name.

**C3. A library config can't be nested in an app config with a different `Context`** ✅ · med

- **Scenario:** a library exports `LibConfig` with `Context = ()`, and the app has
  `#[config(context = AppCtx)]` and nests it. This fails with E0308 "expected `&()`, found
  `&AppCtx`".
- **Where:** `internal/merge.rs:35` and `internal/validate.rs:74`, `:101` require `Context = Ctx`.
- **Simplest fix:** `#[setting(nested, context = default)]`, which passes `&Default::default()` for
  the child's context. A mapping fn (`fn(&Parent) -> &Child`) is the bigger version. Ask before
  building, since it changes what `finalize`, `merge` and `validate` pass.

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
