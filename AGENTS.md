# AGENTS.md

Orientation for AI/agent sessions working in this repo. Read this before touching `crates/core` or
`crates/macros`.

## What this project is

Schematic is a layered configuration library. Users write a plain Rust struct/enum, add
`#[derive(Config)]`, and get back:

- A **partial type** (`PartialExample`) where every setting is optional, used for parsing and
  merging config layers (files, env vars, extends chains).
- A **full type** (`Example`) where every setting is populated, produced once all layers are merged
  and finalized.

Everything hard about this codebase comes from that split. If you understand Config↔Partial, the
rest follows.

## Workspace map

| Crate              | Package            | Role                                                                                                           |
| ------------------ | ------------------ | -------------------------------------------------------------------------------------------------------------- |
| `crates/schematic` | `schematic`        | The user-facing runtime: `Config`/`PartialConfig` traits, loader, merge/validate/env helpers, schema renderers. |
| `crates/types`     | `schematic_types`  | `Schema`, `SchemaBuilder`, `SchemaType`, the `Schematic` trait. No macro code.                                  |
| `crates/core`      | `schematic_core`   | **Where the derive lives.** Container/field/variant analysis and all codegen.                                   |
| `crates/macros`    | `schematic_macros` | Thin proc-macro shell over `core`. Exports `Config`, `ConfigEnum`, and `Schematic`.                             |
| `crates/test-app`  | `test_app`         | Manual smoke-test binary.                                                                                       |

### Migration status — read this first

The rewrite has landed. The old hand-written derive was deleted and `crates/macros-next` was renamed
to `crates/macros`, so `schematic_macros` is now a thin shell over `core` and there is only one
derive implementation. Anything describing `macros` as a separate production implementation is
stale.

All three derives declare `serde` as a helper attribute, so `#[serde(untagged)]`, `#[serde(tag)]`,
and friends are legal on a type that derives only `Config`. That is the replacement for the removed
`#[config(serde(...))]`.

Deliberately dropped in the rewrite, and not coming back:

- `#[config(serde(...))]` — write the native `#[serde(...)]` attribute instead.
- `#[variant(value)]` — `#[variant(rename)]` covers it.
- The `tracing` feature. Generated code is no longer wrapped in `#[tracing::instrument]`; the loader
  still is.

Named-field enum variants panicked in the old derive. They're supported now; see "Named variants"
below.

`#[setting(skip_deserializing_if)]` (and the serde one) panics, as serde has no such attribute.
Other attributes that would do nothing for a derive panic in `Container::validate_attributes`, which
`ToTokens` runs first, since `macro_type` is only set after `Container::new`.

`ConfigEnum` has no implementation of its own. It is the same `Container` with
`macro_type = ContainerMacro::ConfigUnitEnum`, which swaps what `ToTokens` renders — the
`ConfigEnum`/`FromStr`/`TryFrom`/`Display` impls plus `impl_schematic_full`. The schema comes out of
the ordinary unit-enum path: `get_tag_format` reports `Unit` for a config enum (even when a
`fallback` variant means it isn't `is_unit_only`), so `impl_schema_type` emits a literal per unit
variant and infers the fallback's inner type as a string, as the old derive did.

A `ConfigEnum` may also hold `#[serde(untagged)]` single-field tuple variants, which is how it models
a literal union like `"auto" | "off" | number`. `FromStr` tries the named values, then each untagged
variant through the inner type's own `FromStr`, then the fallback. Untagged variants parse the input
as written, not after `before_parse`, which would uppercase `true` into something `bool` rejects. Its schema goes through the same `Unit` path as a fallback, so the
non-literal variant sits inside an `EnumType`. Deserializing is still the user's own serde derive,
so a mismatch gets serde's "did not match any variant" message, not the itemized one below.

**Casing has no default.** The old derive applied `camelCase` to struct fields and `kebab-case` to
enum variants. Core applies nothing — a name is used exactly as written, so Rust's `snake_case` is
what ships unless `rename_all` says otherwise. This was the single largest source of churn when the
rewrite landed, and it is why so many fixtures and snapshots in `crates/schematic/tests` are
snake_case. When `rename_all`
is set, `Field::get_name` and `Variant::get_name` apply it via `format_case`, so schemas,
`settings()`, and validation paths agree with what serde accepts. `format_case` is a port of serde's
own `RenameRule`, not `convert_case`: serde splits a variant before every uppercase letter, so
`MacOS` is `mac-o-s` in kebab case, and anything else disagrees with what serde deserializes.
`before_parse` still normalizes input with `convert_case`, which keeps an acronym whole, so a
`ConfigEnum` with it also matches the identifier normalized that way (`format_loose_case`). Env keys
deliberately skip the casing and stay derived from the Rust name (or an explicit `rename`). Raw
identifiers are unrawed everywhere a name is derived, so `r#type` is `type`.

**All three derives support generics**, where the old derive supported them only on `Schematic`.
`Container` carries `input.generics`, and every emitted item threads them
through: the partial declaration, its `Default`/`Deserialize`, `PartialConfig`, `Config`, and both
`Schematic` impls. Three things fall out of that:

- No bounds are added for you. `PartialConfig` requires `Clone + Default + DeserializeOwned +
  Schematic + Serialize`, so a generic `Config` needs those on its own type parameters, and a
  generic `Schematic` needs `T: Schematic`. An under-bounded type fails to compile at the impl.
- The partial gets an explicit `#[serde(bound(deserialize = "T: DeserializeOwned"))]`. Without it
  serde infers `T: Deserialize<'de>`, which is ambiguous against the `DeserializeOwned` the where
  clause carries. `#[config(partial(serde(bound(...))))]` suppresses the generated one.
- A generic type's `schema_name` appends each type argument (`Wrapper<String>` becomes
  `WrapperString`, its partial `PartialWrapperString`), because schemas are keyed by name alone and
  every instantiation would otherwise claim the same one.

A generic `ConfigEnum` only makes sense with a `fallback`, since unit variants carry no data. Its
`Display` writes each arm for itself, so the fallback goes through `T: Display` instead of having to
be a `&str`.

The old hand-written derive is gone from the worktree but still reachable through git history, at
`git show <pre-migration-rev>:crates/macros/...`. It is a useful reference for *intended* behavior
and the shapes users depend on, but it is **not** a correctness oracle — the rewrite fixed a long
list of real bugs in it, several of which only surfaced when `schematic`'s own suite was pointed at
`core`.

## The Config ↔ Partial model

For `struct Example { name: String, port: Option<u16> }` the derive emits:

```rust
// The partial: every setting optional, used for parsing + merging
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields)]
struct PartialExample {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,   // already optional — NOT double-wrapped
}

impl PartialConfig for PartialExample { /* default_values, env_values, extends_from, finalize, merge, validate */ }
impl Config for Example { type Partial = PartialExample; /* from_partial, settings */ }
impl Default for Example { /* via from_partial(default_partial()) */ }
impl Schematic for Example { /* schema_name, build_schema */ }
impl Schematic for PartialExample { /* delegates + partialize_schema */ }
```

Runtime flow: each source is parsed into a partial layer, and the sources are merged in order,
from the first (`None` without any). `internal::layers::apply_layers` then reads `env_values` against that
merge and merges it over, and reads `default_values` against the result and merges *that* under,
then `finalize` runs per-setting transforms and nested finalization. Finally `validate` runs before
`from_partial` produces the full type. Precedence, lowest to highest: defaults → sources → env. A
partial added with `ConfigLoader::partial` (`Source::Memory`) is a source like a file, so it sits
below env too. `ConfigLoadResult.layers` reports the defaults as a `Source::Defaults` layer and the
env as a `Source::EnvVars` layer, each only when it sets something, so each default handler and env
read runs once per load. An optional file that doesn't exist provides nothing, so it isn't a layer.

**The layers are read against `self`.** `default_values(&self, ctx)` and `env_values(&self)` take
the partial they go under or over, which is what lets the layers depend on what the sources picked:

- Env is read **before** defaults, as a tag's variable can pick the variant whose defaults apply.
- Defaults go under everything else merged together — `D ⊕ (S1 ⊕ … ⊕ E)`, not `((D ⊕ S1) ⊕ …) ⊕ E`.
  An enum's merge replaces another variant, so a left fold drops the defaults of a variant that a
  later source or the tag picks, as an earlier source picking another replaces them. Every merge
  but an enum's is associative, so for a struct the two are the same.
- A missing partial is **viewed** as its `Default`, but never merged, as merging an enum's
  `Default` (its default variant, holding type defaults) over its defaults would replace the
  `default = value` they hold. That's why the sources are folded from the first.

**Defaults and env are applied once, as the outermost config's layers.** A config's
`default_values` and `env_values` contain those of its nested configs, so `finalize` runs
transforms and finalizes each nested config *without* applying its own defaults or env. Per kind of
nested setting, through `self`:

- **Bare** (`child: Child`): `internal::layers::nested_default_values(self.child.as_ref(), ctx)` and
  `nested_env_values(..)`, which view a missing config as its `Default`, as the full type always
  holds one.
- **Optional** (`Option<Child>`): defaults only when `self` holds one, so a parent's defaults leave
  a missing one `None`. Env is read as for a bare one, so a variable can provide it. A recursive one
  (`Option<Box<Self>>`) reads no env, or a missing one would recurse forever.
- **Enums** return the layers of the variant `self` holds: a named variant's fields' defaults and
  env, a tuple variant's nested values', and the default variant's `default = value`. A variant
  with nothing to default has no arm (`None`), except the default variant, which is held as it is.
  The tag's variable (`impl_partial_env_values_tag`, through `EnvManager::pick_variant`) picks a
  variant unless `self` already holds it (`std::mem::discriminant`), so a source's values within
  it stand, and the fields are then read from whichever variant that leaves.
- **Collection items** are the one boundary: no layer holds their defaults or env, as a collection
  replaces on merge, so each item goes through the hidden `internal::layers::finalize_with_layers`, which
  runs `apply_layers` on the item, as the loader does on a config, then finalizes. They aren't
  reported in `layers`.
- **A tuple variant holding several values**, or a recursive or collected config, reads its nested
  values' env in `finalize`, through the hidden `internal::layers::finalize_with_env`
  (`Variant::has_layered_env`). An env layer would have to restate every value it didn't read,
  which would replace a source's plain value or a nested enum's variant, as neither has an empty
  form to merge. Its defaults are still in the layer, as they go *under* the source.
- `partial.finalize(ctx)` on its own applies no defaults or env, save the case above.

A struct or variant whose every default may be `None` (nested configs, handlers) returns `None`
when they all are (`impl_partial_defaults_if_any`), so `layers` never reports defaults that set
nothing. `Config::default_partial` is the provided method only —
`Partial::default().default_values(&ctx)` — as `&self` already recurses into bare nested configs,
the default variant's values, and named variant fields.

Key invariants:

- A setting's **outer `Option` doubles as the partial's `Option`** — `Option<T>` becomes
  `Option<T>`, not `Option<Option<T>>`.
- **Tuple variant payloads get no synthetic `Option`** — `A(bool)` stays `A(bool)`. A **named
  variant's fields do**, as each is a setting (`A { a: Option<bool> }`); see "Named variants".
- **Wrapper types (`Box`/`Arc`/`Rc`) are stripped from the partial** and re-applied in
  `from_partial`. This is why partials don't need serde's `rc` feature. Exceptions: kept when the
  inner type is unsized (`Box<str>`, `Box<Path>`, `Box<[T]>`, …), and a `Box` is kept around a
  nested config that holds the container itself (`Option<Box<Self>>`, `Value::keep_box`), as the
  partial would otherwise have an infinite size. `Self` within a setting is replaced by the
  container's type before analysis, as within the partial it would mean the partial.

## The derive pipeline in `core`

Entry point is `Container::new(DeriveInput)` in `crates/core/src/container.rs`, which returns a
`darling::Result`. An attribute that fails to parse, on the container, a field, a variant, or a
variant's value, is collected rather than unwrapped, and the macro shell reports them all with
`write_errors()`, each spanned to the attribute. `Container::from` is the same, panicking with the
errors' message, which is what the snapshot tests call. A misused attribute that parses still
panics (see "Decided semantics").

```
Container  (the struct/enum)          container.rs
├─ ContainerInner::NamedStruct   { fields: Vec<Field> }
├─ ContainerInner::UnnamedStruct { fields: Vec<Field> }   // tuple struct
└─ ContainerInner::Enum          { variants: Vec<Variant> }  // unit, tuple, and named, mixed

Field    (a struct setting)           field.rs    → FieldValue   → Value
Variant  (an enum variant)            variant.rs  → VariantValue → Value  (tuple values)
                                                  → Field                (named fields)
Value    (type analysis + walkers)    value.rs
```

An enum is one kind, because its variants may be of any kind at once. What depends on them is
asked of the variants, never stored beside them: `ContainerInner::is_unit_only` (an enum serde
writes as strings, so tagged `Unit`, with a string-enum schema) and `has_named_variants` (an enum
with settings, so it denies unknown fields and accepts `allow_unknown_fields`). Everything else is
per variant, so a unit variant adds no arm to `finalize` or `validate` and is reported by its name
in `settings()`, whichever enum holds it.

`ToTokens for Container` emits the whole derive, in this order:

```rust
impl_partial_type()             // the partial struct/enum declaration
impl_partial_type_default()     // manual Default for partial enums
impl_partial_type_deserialize() // try-each-variant Deserialize for untagged enums
impl_partial()                  // impl PartialConfig
impl_full()                     // impl Config + impl Default
impl_schematic()                // impl Schematic for both types
```

Each piece is separately callable, which is what the snapshot tests do.

### Naming conventions

- `impl_partial_*` → code for the partial type / `PartialConfig`.
- `impl_full_*` → code for the full type / `Config`.
- `get_*` → derive-time values (names, keys, attributes), not codegen.
- Most codegen methods return `ImplResult { value, no_value }`. `no_value` means "emit nothing"
  (the caller skips the row/statement).
- Generated code calls the runtime's hidden helpers by their full path, like
  `schematic::internal::merge::MergeManager::new`, never through a glob import, as one in a
  generated body would shadow the names a user's `default`, `merge`, or `validate` expression
  refers to. `crates/schematic/src/internal` is split by domain (`layers`, `env`, `merge`,
  `validate`, `defaults`, `transform`, `untagged`, `settings`, `config_enum`, `schema`), and its `mod.rs` only
  declares them, so a new helper goes in the module of its domain rather than being re-exported.
  `internal` exists under `config` or `schema`, as a `Schematic`-only derive's `build_schema`
  calls `internal::schema`, so every other module is gated on `config` itself.
  The one import is for a string default, whose `DefaultString` traits must be in scope for
  autoref to pick one, so they're imported `as _` within the value's own block.
- What doesn't depend on the derive belongs in a helper, with generated code supplying only what
  the derive knows: keys, names, types, and the arms that build a variant. The managers each end
  with `finish`, `EnvManager::pick_variant` builds the tag's error from the names it's handed,
  and an untagged `Deserialize` is a run of `UntaggedManager` attempts (`variant`, `unit`,
  `literal`) ending in `into_error`, so the generated code never formats a message or holds the
  errors itself. Likewise `ValidateManager::missing_nested` (and its `_with`/`flattened` forms)
  checks `finalizing` for a missing bare nested config, `internal::schema` wraps a tagged
  variant's schema (`tagged_unit`, `internally_tagged`, `adjacently_tagged`) and builds the
  partial's (`build_partial_schema`), and `config_enum::parse_fallback` converts a `ConfigEnum`'s
  fallback. A default or transform that fails is converted with the setting's path, which the
  generated code passes (`defaults::handle_default_result`, `parse_default`,
  `transform::handle_transform_result`), from `Field::get_error_path`: the serde name, or
  `Variant.field` for a named variant's field, as validation reports it. Only the setting's own
  name, not its parent's, as a nested config's are generated without them.

### `Value` and the layer system

`Value::new` decomposes a type into ordered `layers` plus an inner type:

```
Arc<Vec<Option<NestedConfig>>>  →  layers: [Arc, Vec, Option], inner: NestedConfig
```

- `layers` — every layer, full fidelity. Used to _reconstruct_ the final type.
- `get_partial_layers()` — layers minus wrappers. This is the partial's view, and what
  merge/validate/finalize/default operate on.
- `is_outer_option_wrapped()` — computed on the _partial_ view, so `Box<Option<T>>` correctly
  collapses to one `Option`.
- `strip_wrappers` — false when a wrapper holds an unsized type.
- `keep_box` — true for a nested config holding the container itself, which keeps its `Box`.

Collections are rebuilt with `.into_iter().map(..).collect()`, never by naming the collection type,
so any `FromIterator` collection works and a fully qualified path resolves. A layer that isn't a
known collection (`Layer::Unknown`) panics at derive time when it has to be rebuilt, rather than
emitting `Default::default()` and silently emptying it. Nested configs are always called through
`<T as schematic::Config>` and `<<T as Config>::Partial as PartialConfig>`, never `PartialT`, so
they resolve by the path they were written as.

A layer is known by its name (`get_layer`: `*Vec`, `*Set`, `*Map`), and the type it holds by its
position (`get_item_type_argument`), never as its last argument: a map's value, as a hasher may
follow it, and otherwise the first type argument, as an allocator or a const capacity may
(`ArrayVec<T, CAP>`). A collection of an array holds the array's items (`SmallVec<[T; N]>`), which
`map_type_arguments` walks into when replacing or stripping types within it.

Codegen walks these layers to build nested expressions. Each walker composes inside-out, so position
is respected without special-casing:

| Walker                         | Direction                              |
| ------------------------------ | -------------------------------------- |
| `impl_full_from_partial_value` | Partial → full. _Constructs_ wrappers. |
| `impl_partial_finalize_nested` | Within the partial. Skips wrappers.    |
| `impl_partial_merge_nested`    | Within the partial. Skips wrappers.    |
| `impl_partial_validate_nested` | Within the partial. Skips wrappers.    |

`Vec<Arc<Nested>>` wraps each item; `Arc<Vec<Nested>>` wraps the whole loop — both fall out of the
same code.

The first two rebuild a value through its layers, and share one helper for it, `map_layers` in
`value.rs`, which each calls with a list of `LayerStep`s. A step's `unwrap` says whether the value
already holds that wrapper (the partial kept it, so it's unwrapped and rebuilt) or not (the partial
stripped it, so it's only constructed), and its `ty` makes mapping fallible: collections then
collect into a `Result` of that type, as finalizing can fail. Add a layer kind there, once, rather
than in each walker. Merging and validating don't rebuild anything, so they walk on their own.

### Named variants

A named variant (`File { path: String }`) holds `Variant::settings: Vec<Field>`, while a tuple
variant holds `Variant::values`. Each field is a full `Field`, so a named variant reuses the struct
codegen instead of growing its own: the field carries a `VariantScope` with the variant's casing
and the bindings it's destructured into (`pa` for this partial, `na` for the next). `Field`'s
codegen reaches a setting through `get_access`/`get_next_access`/`get_ref_access`, which are
`self.key`-style for a struct and the binding for a variant, so a struct's output is unchanged.
Bindings are positional (`get_binding_idents`), so a field named `context` or `validate` never
shadows a generated local.

- **Merge**: same variant → each field merged as a struct's (`MergeManager` chain), other → replace.
- **Validate**: the arm calls the hidden `ValidateManager::named_variant(name, |validate| ..)`, which
  joins the variant onto the path, so the struct's field statements report `Name.field` unchanged.
- **Untagged / literal union**: an attempt deserializes a local `struct Variant` that stands in for
  the variant, with the container's generics (plus a `PhantomData` marker when generic), the
  partial's deny and bound, and only the fields' serde attributes (`get_partial_deserialize_attributes`),
  as a derive's own `partial(...)` attribute wouldn't exist there.
- **Schema**: a struct of the fields, marked partial like a nested value for a `Config`, so the partial
  schema makes its fields optional; never marked for a `Schematic`-only derive.
- **Defaults and env**: `default_values` and `env_values` each have an arm per variant, reading the
  fields with the struct codegen through the arm's bindings, so a nested field is viewed through
  `pa` (see the runtime flow above). Only the fields whose layers read their value are bound
  (`Field::defaults_read_value`, `Field::env_reads_value`), as the rest would be unused. A tagged
  enum's `env_values` also picks a variant, from the key its tag derives
  (`impl_partial_env_values_tag`), whose fields are then read. An enum accepts `env_prefix` with a
  named variant, or with a tag that can pick a variant (`Variant::is_env_selectable`), and then sets
  `HAS_ENV_PREFIX`, so a parent's prefix reaches its fields and its tag.
- **Finalize**: only transforms and nested finalization, as a struct's.
- Variant-level `nested`, `context`, `merge`, `required`, `transform`, `validate`, and a `default`
  value panic, as the fields hold them; `extend` panics on a field, as only a struct extends.

## Testing

### The critical caveat

**Core's snapshot tests never compile the generated code.** Each test calls one codegen method in
isolation and snapshots the pretty-printed tokens. Type errors, missing trait bounds, and
cross-method inconsistencies are invisible to them. Several real bugs in this codebase sat in green
snapshots for exactly this reason.

`crates/macros/tests/` is the antidote: it derives against the real runtime and asserts on
behavior, so it catches what snapshots can't. `crates/schematic/tests/` now does too, since it
compiles against `core`. Run both for any `core` change that alters codegen.

```bash
cargo test -p schematic_macros
cargo test -p schematic
```

Snapshot tests (starbase_sandbox/insta):

```bash
cargo test -p schematic_core                       # run
INSTA_UPDATE=always cargo test -p schematic_core   # regenerate snapshots
```

Always **read** regenerated snapshots. A fresh snapshot captures whatever the code did, correct or
not — it is not verification on its own.

### Verifying generated code actually compiles

For any codegen change, compile the output against the real runtime. The recipe:

1. Write a temporary test in `crates/core/tests/` that builds `Container::from(...)` for your
   fixtures, prints `container.to_token_stream()` through `pretty()`, and brackets the output with
   markers.
2. Pipe that into a scratch crate's `src/lib.rs`, alongside hand-written base types (the derive only
   emits impls, not the base struct) and any helper fns the fixtures reference.
3. Point the scratch crate at `crates/schematic` by path, copy the workspace `Cargo.lock`, and set
   `CARGO_TARGET_DIR` to the workspace `target` so deps are cached.
4. `cargo check`, then add runtime assertions and `cargo test -- --test-threads=1` (env-var tests
   share a process).
5. Delete the temporary test file when done.

This is the only thing that catches type errors, and it has repeatedly found bugs that snapshots
could not.

### Other suites

```bash
cargo test -p schematic          # runtime + old-derive integration tests
cargo test --workspace           # what CI runs
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

`cargo clippy --workspace --all-targets` is clean, and so is `cargo build -p schematic` on its own.
If you see a warning now, it is yours.

Those two are not the same check. `--workspace --all-targets` unifies features across crates, and
dev-dependencies turn nearly everything on, so a re-export or `mut` that is dead under a narrower
feature set stays invisible there. Build individual crates, and a few feature combinations, before
claiming a change is warning-free:

```bash
cargo build -p schematic                                          # default features only
cargo build -p schematic --no-default-features                    # nothing on
cargo build -p schematic --no-default-features --features config  # one feature at a time
```

The same unification is why `cargo test --workspace` can pass while a real feature combination
fails to compile — see the note about `schematic_types` below.

Also run `cargo test -p schematic_types` on its own. `cargo test --workspace` unifies features across
crates, so a crate whose own feature list is incomplete still compiles there — that hid a broken
`serde` feature in `types` until it was audited.

Also verify with **no features** (`cargo build -p schematic_core`) — much of the codegen is behind
`env`/`extends`/`schema`/`validate` cfgs, and dev-deps enable them all, so feature-gated mistakes
only show in a bare build.

## Decided semantics — do not "fix" these

These were deliberated and settled. If something looks wrong, it probably isn't.

- **Env keys.** Explicit `#[setting(env = "KEY")]` is absolute — a prefix never applies to it. Only
  keys _derived_ from a setting name get the prefix, and only when the container declares
  `env_prefix`. Deriving keys for every container would force `FromStr` on every setting, which
  breaks `Duration`, tuples, etc. A parent's `#[setting(nested, env_prefix)]` therefore only
  applies to a child that declares one, and it *adds* a prefix rather than replacing the child's.
  `EnvManager` reads the child's own prefix first and the parent's second, and that order is what
  makes the child's own prefix win: a nested config doesn't re-read the environment in
  `finalize`, so the parent's env layer, read through `EnvManager`, is the only read. (The
  old replace-the-prefix design let the parent win.) Making the parent win would need the order
  flipped, which the maintainer rejected. `settings()`
  and `SchemaField.env_var` report the derived key with the container's *own* prefix
  (`Field::get_env_var_name`), because the parent's is only known at runtime and the own key always
  works. A setting that never reads the environment, like a collection without `parse_env`, reports
  nothing. A parent's prefix reaches an enum with an `env_prefix` of its own, its named variants'
  fields and its tag alike, as an enum's env is a layer like a struct's, so it sets
  `HAS_ENV_PREFIX`. It never reaches a collection, whose items aren't in the layer. An `Option<Enum>`
  that no source provided reads its default variant's env, and a variable makes it `Some`, as for an
  `Option<Struct>`. A tagged enum's `env_prefix` also derives a key from its tag
  (`Container::get_env_tag_key`, `tag = "type"` → `TYPE`), which picks a variant by its serde name
  or an alias, exactly, and is reported as the tag's `SchemaField.env_var`. By decision an
  externally tagged enum gets no such key (there's no tag, and no attribute to name one), and each
  item of a collection reads it, so it retypes every item, consistent with items reading env.
- **Nested collections replace by default** on merge; bare nested configs merge recursively. Supply
  `merge` to change it — `merge::merge_partials` merges a map of nested configs by key.
- **Nested tuple variants support multiple values**, each position handled per its own shape (merge,
  replace, or manager-wrapped).
- **`required` works on any struct setting, and still needs `Default`.** A required setting is left
  out of `default_values`, so its partial stays `None` until a source or env provides it, and the
  finalizing `validate` pass reports it. `from_partial` keeps `unwrap_or_default()`, because it is
  public and infallible, `impl Default` is emitted for every config, and the check vanishes without
  the `validate` feature — dropping the `T: Default` bound would need all three revisited. A bare
  nested setting that no layer provided is validated as an empty partial when finalizing, since
  `from_partial` builds one anyway; without that, a child whose `default_values` is `Ok(None)` hid
  its required settings.
- **A variant's `default` value lives in `default_values`, not the partial's `Default`.**
  `#[setting(default = value)]` on a variant holding a single value marks it as the default and is
  what `default_values` puts in it when the partial holds that variant
  (`Variant::impl_partial_default_values`), which is what the full type's `Default` goes through. A
  default variant holding a bare nested config holds that config's defaults there, read through it.
  The partial's own `Default` holds the type default, like the empty
  partial of a struct, since it has no context for a handler and can't return a string conversion's
  error. A unit variant's `default = bool` still only marks it, as it always has.
- **`Config::default_partial` panics** on default-value errors. `Default::default()` can't return a
  `Result`, and silently returning wrong values is worse.
- **Validation error paths use serde names** (renames honored) for fields and variants; variants use
  `PathSegment::Variant` + index, rendering `Many[0].inner`.
- **A `#[setting(default)]` path is told apart by its casing.** `LevelFilter::Debug` names a value,
  `find_unused_port` names a handler function, and both arrive as a bare `Expr::Path` that nothing at
  derive time can resolve. `path_names_a_value` goes by the last segment's first character, which
  works because a name breaking Rust's conventions would already be earning a lint. Non-literal
  defaults still never reach the schema — `impl_schema_type` only reads `Expr::Lit`.
- **Partial fields are always `pub`**, never the setting's own visibility. The partial is what a
  caller receives from the loader, so a `pub(crate)` setting would otherwise leave them a field they
  cannot read or set through `..Default::default()`. The partial *type* still inherits the
  container's visibility, which is what actually caps how far those fields reach.
- **A named variant's fields are settings**, optional in the partial and merged field by field,
  rather than values written in full like a tuple variant's. Its fields' defaults apply whenever the
  variant is picked, as the enum's `default_values` is read against the variant the partial holds
  (see the runtime flow). Names take the variant's `rename_all`, else the enum's `rename_all_fields`, never the
  enum's `rename_all`, which names the variants.
- **An enum with a named variant denies unknown fields**, as a struct does, through serde's
  enum-wide `deny_unknown_fields`, so it also applies to the keys around an internally or adjacently
  tagged variant. `#[config(allow_unknown_fields)]` is accepted on such an enum, and a flattened map
  in any named variant turns the denial off for all of them.
- **A container `#[serde(default)]` makes every field optional**, the same as one declared on the
  field, since serde falls back per field when the input omits it. `SchemaField.optional` reflects
  that, which is what puts the `?` on a TypeScript property.
- **An `Option` field is optional in the schema**, as serde fills in an omitted one with `None`, and
  a `#[setting(required)]` one is described by the type it holds, so neither nullable nor optional
  (`Field::get_schema_type`). There's no `SchemaField.required`; `optional: false` says it.
- **`StructType.allow_unknown_fields` defaults to closed.** It's inverted from serde on purpose, so
  hand-written, std, and synthesized variant schemas stay closed. The derive sets it to what's
  deserialized: a config's partial (`#[config(allow_unknown_fields)]`), or the serde attributes of a
  type that only derives `Schematic`.
- **Serde can't deny unknown fields through a flattened field it reads as a map**, as
  `FlatMapAccess` borrows the parent's keys where a plain struct's `FlatStructAccess` takes them. So a
  partial with a flattened map doesn't deny unknown fields (none is unknown), and a parent flattening
  a config whose partial `FLATTENS` fails to compile unless both `ALLOWS_UNKNOWN_FIELDS`
  (`impl_partial_checks`). Making that case load strictly was rejected as too much machinery.
- **Derive-time panics for wrong attribute usage are intentional.** The maintainer wants loud
  failure over silent no-ops. `#[setting(nested)]` on a primitive panics. A panic only points at
  the derive, so one about a setting or variant goes through its `fail` (`Field::fail`,
  `Value::fail`, `Variant::fail`), which prefixes the description `Value::owner` holds, like
  ``Setting `port`: `` or ``Variant `A`: `` (Rust names, a tuple struct's by position). A new panic
  about a setting uses them rather than `panic!`. Turning them into spanned errors through the
  darling accumulator was considered and set aside, as checks made while generating code can't
  return one, so some would stay panics anyway. Only attributes that fail to parse are errors.
- **Error locations are strings, by decision.** A source's own parse or validation error names it:
  its path relative to `set_root`, a code source's file name, or a memory source's name. The final
  validation, of the merged result, names every source and `environment variables` when they set
  something, joined with `, ` (`ConfigLoader::get_merged_location`), as any of them may have set
  what failed, and the config's name with neither. Default values are never listed. Exact blame,
  the one layer that set the failing path, isn't built. `ConfigError` keeps `location: String`,
  as moon builds `ConfigError::Validator` itself, and a `Source` can't hold a list. Without
  `schema`, the config's name is `short_type_name::<T>()`, never `<unknown>`.
- **A `#[setting(default)]` is any expression.** Only a string (converted into the type) and a bare
  path (a value or a handler, by its casing) are special. Anything else is written as is, and the
  compiler checks its type, so there's no list of accepted expression kinds to keep up.
- **Partials keep their `Partial` names in error messages**, like `expected struct PartialChild`.
  serde_derive writes that from the Rust type's name, so `rename` doesn't change it, and
  `expecting` would also replace each enum variant's message and a struct's "with N elements". The
  maintainer chose to leave it.

- **Validators are called through a generated closure**, never passed by value, so that the value
  deref coerces on its way in — a `String` setting reaches a `&str` validator and a `Vec<T>` a
  `&[T]` one. `ValidateManager::check` therefore takes `impl FnOnce(V, &D, &Ctx, bool)` and takes
  the value by move, because a variant of several values passes a *tuple of references*, not a
  reference to a tuple. It returns `Result<(), E: IntoValidateErrors>`, so a validator may report
  several errors while `ValidateResult` stays a single one. `#[config(validate)]` goes through
  `check_config` the same way, without a value. Boxing the validator instead breaks built-ins like `validate::extends_string`.
- **A doc comment keeps the line structure the author wrote.** Renderers depend on it — a JSDoc
  block or a template comment that was several lines has to stay several lines, and collapsing it
  breaks markdown lists and paragraph breaks. A block comment (`/** ... */`) arrives as a single
  multi-line attribute value, so its leading `*` is a continuation marker and gets stripped; a `///`
  line arrives as its own attribute, so a `*` there is markdown and survives.

## Gotchas

- `#[setting(...)]` and `#[serde(...)]` both feed the derive. Setting attrs take precedence; aliases
  from both are merged.
- All three derives declare `serde` as a helper attribute, so `#[serde(...)]` is accepted on a type
  that derives only `Config`. Two derives declaring the same helper is fine — serde's own
  `Serialize`/`Deserialize` pair does it.
- `Schema::partialize()` on a tagged enum variant marks the **value**, never the `{tag, content}`
  struct synthesized around it. Marking the wrapper makes the runtime nullify the tag, so the
  partial schema claims `{"type": null}` is valid. An internally tagged variant can't avoid it, as
  its tag is a field of the value, so `partialize_schema` leaves a `Literal` field alone, treating it
  as the tag, as the Pkl renderer already did.
- Unit enums must stay externally tagged — `#[serde(untagged)]` on a unit-only enum makes variants
  deserializable only from `null`.
- A **literal union** (`Container::is_literal_union`) is an externally tagged `Config` enum whose
  tagged variants are all units and which has at least one untagged variant: `"a" | "b" | boolean`.
  Its partial gets the same hand-written `Deserialize` as an untagged enum, matching the unit names
  (and aliases) as one group, then each untagged variant in order, so the error lists the expected
  names. A tagged variant that holds a value is written as a map, so any such enum keeps serde's
  derive. Serialize stays derived in both cases, which is what keeps a literal union round-tripping.
- With `schema_serde`, every shape struct (and `Schema`, `SchemaField`) has a container-level
  `serde(default)`, so hand-written schema JSON may leave any field out, and field-level attributes
  only carry `skip_serializing_if`. `LiteralType` is the exception, as `LiteralValue` has no default.
  A new field on a shape needs a `Default` that is a valid schema, as an absent one takes it.
- `SchemaField` metadata is derive-time only; `partialize_schema` at runtime is what turns a full
  schema into a partial one.
- A literal `#[setting(default)]` is recorded in `SchemaField.default`, never in the schema of the
  field's type, because a named type's schema is registered once and shared by every field that
  uses it, so a field default there leaks to the others. Renderers read
  `SchemaField::get_default`. For the same reason a nullable union doesn't report the default of a
  named type it holds (`Option<Mode>` has no default), and `partialize_schema` copies a field's
  default onto it before making it nullable.
- `StructType.partial` means "refers to a partial config type", **not** "has been partialized".
  `Schema::nullify()` moves a schema's name into the union variant it creates — surprising when
  asserting on partial schemas.
- `SchemaType` is internally tagged, so any new variant must carry a map-shaped payload. This is why
  `Reference { name, partial }` is a struct variant and not a newtype — serde refuses to internally
  tag a newtype wrapping a bare string, which silently broke every cyclic schema.
- A recursive config resolves its cycle to a `Reference`, and that reference has to be renamed
  alongside the type it points at. `Schema::partialize` sets `Reference.partial`, and
  `partialize_schema` reads it to apply the `Partial` prefix — without both halves the partial
  schema `$ref`s a type that was never rendered.
- `EnumType.values` is a *derived subset* of `EnumType.variants` — variants with a non-literal schema
  (`#[setting(null)]`) contribute no value, so the two differ in length. `default_index` indexes
  `variants`; always read it back through `EnumType::get_default`, never `values[index]`.
- A union's unit variant is a `Literal`, which reports no default of its own, so a renderer reads a
  union's default through `UnionType::get_default` (or `Schema::get_default`), which resolves the
  literal at `default_index`. Don't make `Literal` report its value as a default instead: every
  literal-typed field, including the tag of an internally tagged variant, would claim one.
- `Schema` no longer derefs to `SchemaType` — the accessors it needs (`get_default`, `is_null`,
  `add_field`, …) are forwarded explicitly. Reach for `schema.ty` when you want the type itself.
- `schema_name` must be unique per type; `SchemaGenerator::add` panics on a collision because schemas
  are keyed by name and the loser would silently vanish behind the winner's `$ref`s. Adding the same
  type twice is fine, as is a recursive type registering itself at several depths.
- Before adding a `Schematic` impl, serialize the type and look at the output. A schema describes
  what serde accepts, not what the type looks like — `Duration` and `SystemTime` spent a long time
  modelled as strings while encoding as `{ secs, nanos }`, which made the generated JSON Schema
  reject valid config. `OsString` is why there is still no impl for it: it encodes as
  `{"Unix": [bytes]}`, not a string.
- A parent must reach a nested config through a method a hand-written `PartialConfig` would
  override, not only the hidden ones the derive implements. Nested validation goes through the
  public `validate` (re-pathing its errors), and `internal::layers::nested_env_values` calls `env_values`
  without a prefix, as the hidden `env_values_with_prefix` defaults to it. Don't make a public
  method's default call a hidden one whose default calls it back, as a partial overriding neither
  would recurse.
- A nested config receives its parent's context, as the managers require `Context = Ctx`, unless
  `#[setting(nested, context = func)]` maps it (`Value::context_fn`). Then every call that hands a
  context to the child goes through `func(context)`, and the managers' `*_with` methods take it
  explicitly. Keep the two paths in step: a new call that passes `context` to a nested config must
  use `Value::get_nested_context` / `get_nested_method`, or a mapped config gets the wrong type.
- `StructType.fields` is an `IndexMap` so the *model* keeps declaration order, but renderers iterate
  `StructType::sorted_fields()` so *output* stays alphabetical. Both halves are deliberate: the model
  preserves what the user wrote, the generated file stays stable no matter how the type is ordered.
  A new renderer that iterates `fields` directly will silently reintroduce declaration-ordered output.
- Internally-tagged enums with tuple variants don't compile (serde rejects them). Core doesn't
  currently catch this at derive time.
- A `MergeList` is known by name (`Value::is_merge_list`), like the layers, and merged with
  `merge::merge_list` without a `merge`, which still wins. An alias isn't recognized, by decision.
- Merging must stay associative, `MergeList`'s included. The loader merges the defaults under a
  value the sources and env already merged (`apply_layers`), as does each collection item, so
  `merge` is composition: a `MergeList` holds the index where the previous items go, rather than a
  strategy flag that the next merge would overwrite. An enum's isn't, as another variant replaces,
  which is exactly why the defaults go under rather than first.
- YAML reads an externally tagged variant that holds a value only from a YAML tag (`!file`), never
  the `{ file: .. }` map the other formats, JSON schemas, and templates use. That's serde_norway,
  directly and through a `Value` alike, not the loader's two passes. By decision it's documented,
  recommending internally tagged enums for YAML, rather than read through a deserializer of our own.
- The maintainer's zsh wraps `git` in a `scmpuff` function that breaks in non-interactive shells.
  Use `command git`.

## Docs

User-facing documentation lives in `book/src/` — `config/index.md`, `config/partial.md`,
`config/nested.md`, `config/struct/*.md` (default, env, extend, merge, transform, validate),
`config/enum/*.md`. When changing user-visible behavior, check whether the book states otherwise.
