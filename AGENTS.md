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

| Crate                | Package                 | Role                                                                                                                       |
| -------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `crates/schematic`   | `schematic`             | The user-facing runtime: `Config`/`PartialConfig` traits, loader, merge/validate/env helpers, schema renderers.            |
| `crates/types`       | `schematic_types`       | `Schema`, `SchemaBuilder`, `SchemaType`, the `Schematic` trait. No macro code.                                             |
| `crates/macros`      | `schematic_macros`      | **The production derive.** Currently what ships.                                                                           |
| `crates/core`        | `schematic_core`        | **The rewrite in progress.** Where new work goes.                                                                          |
| `crates/macros-next` | `schematic_macros_next` | Thin proc-macro shell over `core`. Exports `Config` and `Schematic`; `ConfigEnum` is not ported yet.                        |
| `crates/test-app`    | `test_app`              | Manual smoke-test binary using the production derive.                                                                      |

### Migration status — read this first

`crates/macros` is production — it is what `crates/schematic` re-exports, and what users get today.
`crates/core` is a piece-by-piece rewrite of it, consumed by `crates/macros-next`.

`macros-next` exports working `Config` and `Schematic` derives, exercised by real compile-and-run
tests in `crates/macros-next/tests/`. **Nothing depends on it yet** — `crates/schematic` still
re-exports `schematic_macros`, so swapping the two is a separate step blocked on the gaps below.

Still unported, in rough order of what blocks the swap:

- **`tracing`.** The feature is declared and forwarded to `core`, but no codegen reads it. Production
  puts `#[instrument(skip_all)]` on `default_values`, `env_values`, `extends_from`, `finalize`,
  `merge`, `validate_with_path`, `from_partial`, `settings`, `default`, and `build_schema`.
- **`#[setting(skip_deserializing_if)]`** parses but emits nothing (serde has no such attribute).
- `#[setting(exclude)]` is behind `cfg(schema)` in core but unconditional in production, so it fails
  to parse when the feature is off.
- **`#[variant(value)]`** is not carried over. Core uses `#[variant(rename)]`, which the book already
  describes `value` as being "similar to", so the two would be redundant.

Not gaps, despite looking like them: named-field enum variants panic in *both* crates, and
`#[config(serde(...))]` was deliberately removed in the rewrite.

`ConfigEnum` has no implementation of its own. It is the same `Container` with
`macro_type = ContainerMacro::ConfigUnitEnum`, which swaps what `ToTokens` renders — the
`ConfigEnum`/`FromStr`/`TryFrom`/`Display` impls plus `impl_schematic_full`. The schema comes out of
the ordinary unit-enum path: `get_tag_format` reports `Unit` for a config enum (even when a
`fallback` variant makes it structurally unnamed), so `impl_schema_type` emits a literal per unit
variant and infers the fallback's inner type as a string, matching production.

**Casing is a deliberate divergence, not a gap.** Production defaults to `camelCase` for struct
fields and `kebab-case` for enum variants. Core has *no default* — a name is used exactly as
written, so Rust's `snake_case` is what ships unless `rename_all` says otherwise. When `rename_all`
is set, `Field::get_name` and `Variant::get_name` apply it via `format_case`, so schemas,
`settings()`, and validation paths agree with what serde accepts. Env keys deliberately skip the
casing and stay derived from the Rust name (or an explicit `rename`), matching production.

Generics are supported by `Schematic` but not `Config` (the partial type isn't generic), which
matches production. As in production, `#[derive(Schematic)]` on a generic type does not add a
`T: Schematic` bound for you — write it yourself. Unlike production, a generic type's `schema_name`
appends each type argument (`Wrapper<String>` becomes `WrapperString`), because schemas are keyed by
name alone and every instantiation would otherwise claim the same one.

When implementing something in `core`, the old implementation in `crates/macros` is the reference.
It is _not_ always correct — this session found many bugs in it — but it tells you the intended
behavior and the shapes users depend on.

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

Runtime flow: each source is parsed into a partial layer, layers are combined with `merge`, then
`finalize` re-merges that result **over** `default_values` and **under** `env_values` (so env wins),
runs per-setting transforms and nested finalization, and finally `validate` runs before
`from_partial` produces the full type. Precedence, lowest to highest: defaults → files → env.

Key invariants:

- A setting's **outer `Option` doubles as the partial's `Option`** — `Option<T>` becomes
  `Option<T>`, not `Option<Option<T>>`.
- **Enum variant payloads get no synthetic `Option`** — `A(bool)` stays `A(bool)`.
- **Wrapper types (`Box`/`Arc`/`Rc`) are stripped from the partial** and re-applied in
  `from_partial`. This is why partials don't need serde's `rc` feature. Exception: kept when the
  inner type is unsized (`Box<str>`, `Arc<str>`, `Box<[T]>`).

## The derive pipeline in `core`

Entry point is `Container::from(DeriveInput)` in `crates/core/src/container.rs`.

```
Container  (the struct/enum)          container.rs
├─ ContainerInner::NamedStruct   { fields: Vec<Field> }
├─ ContainerInner::UnnamedStruct { fields: Vec<Field> }   // tuple struct
├─ ContainerInner::UnnamedEnum   { variants: Vec<Variant> }
└─ ContainerInner::UnitEnum      { variants: Vec<Variant> }  // all variants are units

Field    (a struct setting)           field.rs    → FieldValue   → Value
Variant  (an enum variant)            variant.rs  → VariantValue → Value
Value    (type analysis + walkers)    value.rs
```

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
- Most codegen methods return `ImplResult { value, no_value, requires_internal }`. `no_value` means
  "emit nothing" (the caller skips the row/statement); `requires_internal` means the generated
  method needs `use schematic::internal::*;`, which the container adds via
  `ImplResult::impl_use_internal`.

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

## Testing

### The critical caveat

**Core's snapshot tests never compile the generated code.** Each test calls one codegen method in
isolation and snapshots the pretty-printed tokens. Type errors, missing trait bounds, and
cross-method inconsistencies are invisible to them. Several real bugs in this codebase sat in green
snapshots for exactly this reason.

`crates/macros-next/tests/` is the antidote: it derives against the real runtime and asserts on
behavior, so it catches what snapshots can't. Run it for any `core` change that alters codegen.

```bash
cargo test -p schematic_macros_next
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

Known pre-existing clippy warnings (not yours): a `count` loop counter in `core/src/variant.rs` and
`macros/src/config/variant.rs`, and a collapsible `if` in `macros/src/utils.rs` and
`schematic/src/schema/renderers/template.rs`.

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
  overrides a child that declares one. `settings()` reports explicit keys only.
- **Nested collections replace by default** on merge; bare nested configs merge recursively. Supply
  `merge` to change it.
- **Nested tuple variants support multiple values**, each position handled per its own shape (merge,
  replace, or manager-wrapped).
- **`Config::default_partial` panics** on default-value errors. `Default::default()` can't return a
  `Result`, and silently returning wrong values is worse.
- **Validation error paths use serde names** (renames honored) for fields and variants; variants use
  `PathSegment::Variant` + index, rendering `Many[0].inner`.
- **Derive-time panics for wrong attribute usage are intentional.** The maintainer wants loud
  failure over silent no-ops. `#[setting(nested)]` on a primitive panics.

## Gotchas

- `#[setting(...)]` and `#[serde(...)]` both feed the derive. Setting attrs take precedence; aliases
  from both are merged.
- Unit enums must stay externally tagged — `#[serde(untagged)]` on a unit-only enum makes variants
  deserializable only from `null`.
- `SchemaField` metadata is derive-time only; `partialize_schema` at runtime is what turns a full
  schema into a partial one.
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
- `StructType.fields` is an `IndexMap` so the *model* keeps declaration order, but renderers iterate
  `StructType::sorted_fields()` so *output* stays alphabetical. Both halves are deliberate: the model
  preserves what the user wrote, the generated file stays stable no matter how the type is ordered.
  A new renderer that iterates `fields` directly will silently reintroduce declaration-ordered output.
- Internally-tagged enums with tuple variants don't compile (serde rejects them). Core doesn't
  currently catch this at derive time.
- The maintainer's zsh wraps `git` in a `scmpuff` function that breaks in non-interactive shells.
  Use `command git`.

## Docs

User-facing documentation lives in `book/src/` — `config/index.md`, `config/partial.md`,
`config/nested.md`, `config/struct/*.md` (default, env, extend, merge, transform, validate),
`config/enum/*.md`. When changing user-visible behavior, check whether the book states otherwise.
