# Self-aware default and env layers

Plan for giving `PartialConfig::default_values` and `env_values` a `&self`, so the loader's layers
can depend on what the sources picked. Written 2026-11-15 on `0.21-enum-env`, after
[enum-env.md](enum-env.md) (PR #218). This is a breaking change, which is fine for 0.21.

**Built 2026-11-15 on `0.21-self-api`**, with the recommendation taken for each of the four
decisions. It differs from the plan below in the ways listed under "As built", most importantly in
the loader's merge order, which the plan got wrong.

## As built

- **Defaults go under the merge of everything else, not first in a left fold.** The plan's
  `[Defaults, ..sources, EnvVars]` folded in order drops a variant's defaults whenever an earlier
  layer picked another: `A_defaults ⊕ B ⊕ A` is `A` without them, as an enum's merge replaces
  another variant. `internal::apply_layers` merges `D ⊕ (S1 ⊕ … ⊕ E)` instead. That's the same for
  every associative merge, which every merge but an enum's is, and is how collection items and
  optional configs already had their defaults merged. Three existing tests fail under the plan's
  order (`applies_defaults_of_a_variant_switched_back_to`, `picks_the_variant_its_tag_names`,
  `replaces_another_variant`).
- **`read_layers` is `apply_layers`**, which also returns the merged partial, as the loader needs
  it, and `finalize_with_layers` is `apply_layers(Some(item))` and `finalize`. Both stay hidden
  (decision 4).
- **A tag naming the variant `self` already holds picks nothing** (`std::mem::discriminant`), so a
  source's values within it stand. Otherwise the env layer held the variant with type defaults,
  which replaced a collection it held.
- **Env through the layer only for a variant whose values can be left out.** A named variant's
  fields are optional, and a single nested config is either read or left out, but a tuple variant
  holding several values has no empty form for the ones it didn't read: a plain value or a nested
  enum's variant would be replaced. So any tuple variant holding several values, not only a mixed
  one, reads its nested values' env in `finalize` (`internal::finalize_with_env`,
  `Variant::has_layered_env`), as do a recursive or collected config (decision 2, widened).
- **A layer that sets nothing isn't reported.** A struct or a non-default variant whose every
  default may be `None` (nested configs, handlers) returns `None` when they all are
  (`impl_partial_defaults_if_any`), and a non-default tuple variant holding one config maps its
  defaults, so an optional config's absence doesn't report an empty defaults layer. This also
  fixed a struct of only bare nested configs without defaults reporting one.
- **A unit default variant still returns itself**, rather than `None` as the table below says, so
  the defaults layer still names it, as before.
- `Option<Enum>` reads its default variant's env (decision 1), and `env_values_with_prefix` stays
  hidden beside `env_values` (decision 3).

## Why

`default_values()` and `env_values()` are static, so they run without knowing what the sources
picked. Today every place where the answer depends on that is a **finalize boundary**: the value's
defaults and env are merged around it inside `finalize`, outside any layer.

| Boundary today                         | Why it can't be in a layer                                         |
| -------------------------------------- | ------------------------------------------------------------------ |
| Named variant fields (#216, #218)      | The layer can't know which variant a source picks                  |
| Tuple variant values holding a config  | Same, and the enum's default variant holds the *type* default      |
| `Option<Child>` a source made `Some`   | The parent's defaults leave it `None`, so its defaults come later  |
| Collection items                       | Collections replace on merge, so a layer can't hold item defaults  |

The first three exist only because the layer can't see `self`. They cost us:

- **Extra machinery:**
  - the defaults and env blocks in `Variant::impl_partial_finalize_named`;
  - the `boundary` path of `Value::impl_partial_finalize_nested`;
  - its `skip_outer_option` defaults branch;
  - the `Config::default_partial` overrides (`Container::impl_full_default_partial`,
    `Variant::impl_full_default_partial`, `get_named_defaults(full)`), which exist because an
    enum's `default_values` must hold a nested config's *type* default to avoid applying its
    defaults twice;
  - and Phase 2's empty-variant trick.
- **Holes in the documented model:** `partial.finalize()` applies a named variant's defaults and
  env, although AGENTS.md says `finalize` applies neither. And `ConfigLoadResult.layers` doesn't
  report them.
- **A parent's `#[setting(nested, env_prefix)]` can't reach an enum.** Its fields are read in
  `finalize`, which takes no prefix, so it's a compile error.

With `&self`, those three move into the layers. Only **collection items** stay a boundary, plus
one small case covered under "What stays".

## The new model

### Trait

```rust
pub trait PartialConfig {
    /// The defaults that go under `self`, the merge of every source and the environment.
    /// An enum returns the defaults of the variant it holds, and an optional nested
    /// config contributes defaults only when it's present.
    fn default_values(&self, context: &Self::Context) -> Result<Option<Self>, ConfigError> {
        Ok(None)
    }

    /// The environment variables that go over `self`, the merge of every source. An enum
    /// reads the fields of the variant it holds, unless its tag's variable picks another.
    #[cfg(feature = "env")]
    fn env_values(&self) -> Result<Option<Self>, ConfigError> {
        Ok(None)
    }

    #[cfg(feature = "env")]
    #[doc(hidden)]
    fn env_values_with_prefix(&self, _prefix: Option<&str>) -> Result<Option<Self>, ConfigError> {
        self.env_values()
    }

    // finalize: transforms, nested finalization, and collection items only
}
```

`&self`, not `Option<&Self>`. A missing nested config is viewed as `T::default()`, through hidden
helpers (below), so the trait stays simple. For a struct, `self` only matters for its nested
settings, so a struct's output is the same as today's static one.

### Loader order

Env is read before defaults, because a tag's variable can pick the variant whose defaults apply.
Precedence doesn't change, as the layers are still **merged** defaults → sources → env:

```rust
let sources = self.parse_into_layers(..)?;                        // unchanged
let folded = fold(&sources, context)?;                            // Option<T::Partial>
let layers = internal::read_layers(folded.as_ref(), context)?;    // { defaults, env }

// [Defaults?, ..sources, EnvVars?], reported exactly as merged
let partial = fold(&all_layers, context)?.unwrap_or_default().finalize(context)?;
```

`internal::read_layers(view: Option<&T>, ctx)`:

1. `let mut view = view.cloned().unwrap_or_default();`
2. `let env = view.env_values()?;`, then merge it into `view`. The view now holds the final variant
   and the final `Some`-ness of each optional config.
3. `let defaults = view.default_values(ctx)?;`
4. Return `{ defaults, env }`.

**Fold from the first layer, not from `T::Partial::default()`.** Today `merge_layers` starts from
`Default`. Under the new order that would let an enum's type default beat its default value. Take
`#[setting(default = "auto")] Named(String)` with no sources. The defaults layer holds
`Named("auto")`, but if the fold starts from `Default`, `Named("")` is merged over it and wins, as
plain tuple values replace. Folding from the first layer (`None` when there are none) is equivalent
for every other shape, as `Default ⊕ x == x` for structs, named variants, plain and `MergeList`
values. Do it everywhere layers are folded.

### Collection items

`internal::finalize_with_layers` keeps its job, using the same helper:
`read_layers(Some(&item))`, fold `[defaults, item, env]`, then `finalize`. The loader and items now
share one algorithm, so name it once and use it in both.

### Codegen, per kind of setting

| Setting                           | `default_values(&self)`                                   | `env_values(&self)`                                               |
| --------------------------------- | --------------------------------------------------------- | ----------------------------------------------------------------- |
| Plain                             | As today                                                  | As today                                                          |
| Bare nested `child: Child`        | `nested_default_values(self.child.as_ref(), ctx)`         | `env.nested(nested_env_values(self.child.as_ref(), prefix)?)`     |
| `Option<Child>`                   | only `if let Some(child) = &self.child`                   | as bare, so env can still make it `Some`, as today                |
| Collection of configs             | skipped (items are a boundary)                            | skipped                                                           |
| `Option<Box<Self>>` (`keep_box`)  | only when `Some`, so recursion ends                       | skipped, as today                                                 |
| Enum: unit variant                | `None`, as it holds nothing (the fold falls back to it)   | tag selector only                                                 |
| Enum: named variant               | `Self::A { field defaults, through ref bindings }`        | `Self::A { field env }` for `self`'s or the tag's variant         |
| Enum: tuple, every value nested   | `Self::V(nested defaults, unwrapped)`                     | `Self::V(nested env, unwrapped)`                                  |
| Enum: tuple with a plain value    | `Self::V(nested defaults, default value or type default)` | see "What stays"                                                  |

The hidden helpers in `schematic::internal` take a `context = func` mapping like every nested call:

```rust
pub fn nested_default_values<T: PartialConfig>(value: Option<&T>, context: &T::Context) -> Result<Option<T>, ConfigError>;
pub fn nested_env_values<T: PartialConfig>(value: Option<&T>, prefix: Option<&str>) -> Result<Option<T>, ConfigError>;
```

**Enum `env_values`** becomes one match:

1. Read the tag's variable, if any.
2. Pick the variant it names, or `self`'s.
3. Read that variant's fields, or its nested values.

`Some` when anything was found. The selector returns the variant **with** its env, so Phase 2's
empty-variant trick goes away. `EnvManager::new(prefix, own)` takes the parent's prefix, so
**`HAS_ENV_PREFIX` is set on an enum with `env_prefix`**, and the parent-prefix compile error for
enums goes away.

**`Config::default_partial`** becomes the provided method only:
`read_layers(None).defaults.unwrap_or_default()`, without env. Recursing through `&self` already
reaches bare nested configs, the default variant's nested values, and named variant fields, so
every derive override is deleted.

## What stays

- **Collection items** are the one real boundary. Their defaults and env still go through
  `finalize_with_layers`, and `layers` still doesn't report them.
- **A tuple variant holding a nested config alongside a plain value**, like `Both(Child, bool)`.
  Its defaults can go in the layer (a plain position holds the variant's default value or its type
  default, as the default variant's does today). Its env can't: the env layer would have to restate
  the source's `bool`, which would mis-report provenance, and a `MergeList` position would merge
  into itself and duplicate. **Recommended:** its nested values read their env in `finalize`, the
  only env read left there. See decision 2.
- The "bare nested setting no layer provided is validated as an empty partial" rule in `validate`.
  A child without defaults is still `None` after the defaults layer.
- `MergeList` associativity, which collection items still rely on.

## What's removed

- **In `variant.rs`:**
  - `impl_partial_finalize_named` loses its defaults and env blocks, so it's the struct's
    transform and nested statements in a match arm. With them go `get_env_binding_ident` and the
    `get_named_env_values` merge plumbing; the per-field env codegen moves into `env_values`.
  - `get_named_defaults(full)`, `impl_full_default_partial`, and the `boundary = true` call in
    `impl_partial_finalize` for all-nested tuple values.
- **In `value.rs`:** `impl_partial_finalize_nested`'s `boundary` parameter and its
  `skip_outer_option` defaults branch. Only collections call `finalize_with_layers` now.
- **In `container.rs`:**
  - `impl_full_default_partial`;
  - the enum special case in `impl_partial_default_values` ("type default of a nested config");
  - `impl_partial_env_values_tag`, which folds into the enum's `env_values`;
  - the enum exception in `impl_partial_checks`' parent-prefix message.
- **AGENTS.md:** the "Runtime flow" section collapses. It becomes defaults → sources → env as
  layers, computed against the sources, with collection items as the boundary.

## Behavior changes

Each one goes in the changelog:

1. **Signatures:** `default_values(&self, ctx)`, `env_values(&self)`,
   `env_values_with_prefix(&self, prefix)`. A caller writes `T::default().default_values(&ctx)`. A
   hand-written partial adds `&self` (`hand_written_nested` in `config_test.rs` is the only one in
   the repo; moon's `crates/` calls none of these directly as of 2026-11-15).
2. **`partial.finalize()` never applies defaults or env**, now including named variants'. A caller
   uses the loader, or `internal::read_layers`, if that's made public (decision 4).
3. **`ConfigLoadResult.layers` reports more:**
   - the picked variant's defaults in `Source::Defaults`;
   - its fields' env in `Source::EnvVars`;
   - an optional nested config's defaults when a source made it `Some`.

   Collection items are still not reported.
4. **The loader reads env before running default handlers.** Precedence is unchanged; only the
   order of side effects differs.
5. **A parent's `#[setting(nested, env_prefix)]` reaches an enum** (#218 made that a compile error).
6. **`Option<Enum>` left `None` by the sources** now reads its default variant's field env, and
   becomes `Some` when one is set, as an `Option<Struct>` already does. Today it stays `None` unless
   the tag's variable picks something. See decision 1.

## Decisions to confirm

1. **`Option<Enum>` and the default variant's env (change 6).** Recommended: yes, for consistency
   with structs. The alternative is a special case in `nested_env_values` that only runs the tag
   selector for a missing enum, which needs a way to tell an enum apart (a hidden const).
2. **Env for a tuple variant mixing a nested config and a plain value.** Recommended: keep reading
   its nested values' env in `finalize`. The alternative puts it in the layer, restating the
   source's plain values, and panics at derive time on a `MergeList` beside a nested config. That
   removes the last env read from `finalize`, at the cost of the provenance it reports.
3. **Keep `env_values` + hidden `env_values_with_prefix`.** Recommended: keep the split, so a
   hand-written partial implements the public method alone. Folding them into one
   `env_values(&self, prefix)` saves a method but makes every hand-written partial take a prefix it
   ignores.
4. **Public `read_layers`?** Recommended: keep it hidden, like `finalize_with_layers`, until a
   caller needs to finalize a partial outside the loader.

## Implementation, one commit each

1. **Signatures and loader, no behavior change.** Change the trait and the loader order, and add
   `read_layers`, the nested helpers and the fold from the first layer. Generate `&self` everywhere,
   with structs passing their nested settings through the helpers. Enums ignore `self` for now,
   doing exactly what they do today. Update `hand_written_nested`. **Gate:** every runtime test
   passes unchanged, and the snapshots differ only in signatures and helper calls. Read them all.
2. **Optional nested defaults through `self`.** Remove the `skip_outer_option` branch.
   **Gate:** the `applied_once` and `settings_test` optional-nested tests pass.
3. **Enum defaults through `self`.** Named variants, then tuple variants. Remove the finalize
   defaults block, the `boundary` path for all-nested tuple values, and every `default_partial`
   override. **Gate:** `named_variants`, `variant_default_values`, `mixed_variant_values`, and
   `Default` for every enum fixture.
4. **Enum env through `self`.** Fold the tag selector in, set `HAS_ENV_PREFIX` on an enum with a
   prefix, remove the finalize env block, and make the decision-2 change. **Gate:**
   `named_variant_env` and `env_picked_variant`, plus new tests for a parent prefix on an enum and
   for decision 1.
5. **Docs:** AGENTS.md ("Runtime flow", "Named variants", "Decided semantics → Env keys"), the
   book's `struct/env.md`, `struct/default.md`, `struct/index.md` and `partial.md`, the changelog
   (amend the unreleased G4 and #218 entries rather than stacking new ones), and this file.

## Tests to add

- **Provenance:** `layers` reports a picked named variant's defaults and env, and an optional
  config's defaults (`layers_test.rs`).
- **Parent prefix:** `#[setting(nested, env_prefix = "P_")]` on an enum reaches its fields and its
  tag, alongside the enum's own prefix. The own prefix is read first, as for structs.
- **Fold from the first layer:** `#[setting(default = "auto")] Named(String)` loads `"auto"` with no
  sources, at the root and nested.
- **Once:** `append_vec` + `parse_env`, and a non-idempotent default handler, each applied once on
  a picked named variant, a tuple-nested value, and an optional config.
- **`finalize` alone** applies no defaults or env to a named variant.
- **Decision 1** either way, and **decision 2**'s mixed tuple env.
- The **compile-against-runtime recipe** in AGENTS.md, for each commit that changes codegen. Then
  `cargo build -p schematic_core` with no features, and `schematic` with no features, `config` and
  `env`.

## Risks

- **Recursion:** viewing a missing nested config as `T::default()` and recursing is what the static
  `env_values` already does, so mutual recursion through optional configs is no worse. Keep
  `keep_box` skipped for env, and only recurse into `Some` for defaults.
- **Cost:** the loader folds the sources twice, once for the view and once in the final fold. The
  clones are cheap next to parsing. A collection item folds once, as today.
- **Snapshot churn:** commit 1 touches most `default_values`/`env_values` snapshots in core. Its
  "no behavior change" gate is what makes them reviewable, so keep it separate.
