# Composability

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**C1. Nested configs are referenced by bare identifier** · fixed on `0.21-bugs` and `0.21-compose`

Nested configs are called through `<T as Config>` by the path they're written as, so a config from
another module and `Self` work (`0.21-bugs`). Generic nested configs, like `Wrapper<String>`, and a
type parameter that is a config work too (`0.21-compose`), as the layer walk stops at the first type
that isn't a layer. A type parameter still needs the partial's derive bounds, as std's derives bound
every parameter; see `book/src/config/generics.md`.

**C2. One layer walker instead of three** · fixed on `0.21-compose`

`wrap_layer` no longer names collections (`0.21-bugs`), and `impl_full_from_partial_value` and
`impl_partial_finalize_nested` now share `map_layers`, so a layer kind is handled in one place. The
generated code is unchanged; every core snapshot is identical. Validation still walks on its own, as
it borrows rather than rebuilds.

**C3. A library config can't be nested in an app config with a different `Context`** · fixed on
`0.21-compose`

`#[setting(nested, context = func)]` takes a `fn(&Parent) -> &Child` that maps the parent's context
for every call that passes one to the nested config (defaults, finalize, merge, validate).
`schematic::context::unit` covers a child with `Context = ()`. Code without a `context` is
unchanged; the managers gained `_with` variants that take the child's context.

**C4. A hand-written `PartialConfig` is silently skipped when nested** · fixed on `0.21-compose`

`ValidateManager::nested_at_with` now calls the public `validate` and re-paths its errors, so an
override of either `validate` or the hidden `validate_with_path` is validated when nested. The
hidden `env_values_with_prefix` defaults to `env_values` (and `env_values` to none, with the derive
implementing both), so a hand-written partial's own env is read under a parent `env_prefix`.

**C5. The partial always derives `PartialEq` and `Debug`** · documented on `0.21-compose`

Kept as is, by decision: dropping either would break code that compares or prints partials, or
derives them on a struct holding one. `book/src/config/settings.md` ("Required traits") now states
that every setting's type must implement the partial's derives, and points at the newtype pattern
`RegexSetting` uses.

**C6. Multi-value variants can't mix nested and plain positions** · fixed on `0.21-compose`

`#[setting(nested)]` (and `context`) may be put on a value of a variant, like
`Both(#[setting(nested)] Child, bool)`, which marks that value alone. Each value is merged,
finalized, validated, and given a schema by its own shape. Marking both a variant and one of its
values panics.

**C7. `ConfigLoader` is neither `Send` nor `Sync`** · won't fix, by decision

Adding `+ Send + Sync` to formats and cachers (and `PhantomData<fn() -> T>`) works for every built-in
except `PklFormat`. It stores rpkl's `EvaluatorOptions`, whose client readers are
`Arc<dyn PklModuleReader>` with no `Send` bound, so no `PklFormat` is `Send`, even with default
options. The fix would be `PklFormat::new` taking a `Fn() -> EvaluatorOptions + Send + Sync`
factory, which was declined, so the loader stays as it is. Revisit if rpkl bounds its readers.

**C8. Custom renderers can't reuse built-in helpers** · partly fixed on `0.21-compose`

Built: `SchemaGenerator::render` returns the output as a string; `SchemaRenderer`'s `render_*`
methods default to an "unsupported" error, so a custom renderer implements only what it needs; and
every schema type but `LiteralType` has a container-level `serde(default)` under `schema_serde`. The book's generator page documents custom
renderers, including being generic over the output type `O`.

Not built, by decision: making `split_doc_tags` and the template helpers public, as they'd become
API to keep stable.

**C9. `#[setting(skip)]` fields still require `T: Schematic`** · fixed on `0.21-compose`

A skipped field (`#[setting(skip)]` or `#[serde(skip)]`) is now filtered out of the schema with
excluded ones, so its type isn't inferred or registered. `skip_deserializing` fields stay hidden,
as they're still written.

**C10. Dependencies pulled in without `config`** · fixed on `0.21-compose`

`miette`, `serde`, `serde-content`, `thiserror` and `tracing` are optional, enabled by `config`
(and `miette` by `schema`), and `convert_case` moved from `schema` to the TypeScript and Pkl
renderers. `env`, `extends`, `url` and `validate` enable `config`. A featureless build compiles 22
crates instead of 40, and a schema-only one 29 instead of 42.

Not changed: `validate` still enables garde's `regex`, which `validate::regex` needs. Making it
optional would need a feature of its own that `validate` doesn't include, which breaks any build that
uses `validate::regex` with `default-features = false`.
