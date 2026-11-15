# Environment variables in `Config` enums

Plan for branch `0.21-enum-env`, cut from `develop-0.21` at `a4448e7` (after named variants, #216,
and the `ContainerInner::Enum` merge, #217). See [README.md](README.md) for the legend.

**Goal.** A `Config` enum's settings read environment variables the way a struct's do: `env`,
`parse_env` and a container `env_prefix`, with the same precedence (defaults → sources → env) and
the same metadata in schemas and `settings()`. Picking the variant from the environment is a
separate decision, covered in Phase 2.

## Where things stand

- `env_prefix` on any enum panics (`Container::validate_enum_attributes`, `container.rs:405`).
- `env`, `env_prefix` and `parse_env` on a named variant's field panic
  (`Field::validate_variant_args`, `field.rs:184`).
- An enum's `env_values` is never emitted (`impl_partial_env_values`, `container.rs:1546`), so it is
  always the trait's `Ok(None)`.
- ✅ **Bug from #216** · fixed by Phase 0: a nested config inside a **named** variant never read its environment. Its
  `#[setting(env)]` settings are silently ignored. The same config inside a **tuple** variant does
  read them, because a tuple variant's value goes through `internal::finalize_with_layers`. Probe:

  ```rust
  #[derive(Config)]
  struct Child { #[setting(env = "TOKEN")] token: Option<String> }

  #[derive(Config)]
  enum Auth {
      #[setting(nested)] Tuple(Child),
      Named { #[setting(nested)] child: Child },
  }
  // TOKEN=env, loading {"Tuple": {}} → Tuple(Child { token: Some("env") })
  // TOKEN=env, loading {"Named": {}} → Named { child: Child { token: None } }
  ```

## The constraint that shapes the design

`env_values()` takes no `self`. It runs before any source is merged, so it can't know which variant
the merged result will hold. A partial enum holds exactly one variant, and merging a different
variant replaces it. So an env layer that guessed a variant would override the file's choice
whenever it guessed wrong. Env values for a variant's fields can only be applied once the variant
is known, which means in `finalize`.

That is already how named variants handle **defaults**. `default_values` holds the default variant
with every field empty, and `impl_partial_finalize_named` (`variant.rs:1306`) merges the fields'
defaults under whichever variant was picked. Env is the mirror image: merge the fields' env values
**over** the picked variant in the same arm, after the defaults and the sources. The named variant
becomes a boundary for env, as it already is for defaults, and as collection items and tuple
variant values already are for both.

Every path into an enum then applies its env exactly once:

| Path                             | Enum's env layer (`env_values`) | Fields' env                 |
| -------------------------------- | ------------------------------- | --------------------------- |
| Root (`ConfigLoader<Auth>`)      | `None`                          | `finalize` arm              |
| Bare nested in a struct          | `None` via `env.nested(..)`     | `finalize` arm (via parent) |
| `Option<Auth>` in a struct       | `None`                          | `finalize` arm              |
| Collection item / variant value  | `None` in `finalize_with_layers`| `finalize` arm              |
| Nested in another named variant  | `None`                          | `finalize` arm              |

No env value lives in a layer, so a non-idempotent `merge` (`append_vec` with `parse_env`) can't run
twice. That is the bug G4 fixed, and the plan has to keep it fixed.

**What it costs.** Env values read in `finalize` aren't reported in `ConfigLoadResult.layers`.
Collection items and tuple variant values already work this way. It also means calling
`partial.finalize(ctx)` directly reads env for an enum's variants, as it already applies their
defaults.

## Phase 0: nested configs in named variants read their env · fixed on `0.21-enum-env`

This builds the machinery the next phase reuses, and fixes the ✅ bug on its own.

1. **`Variant::impl_partial_env_values_named`** (new, `variant.rs`). For each setting, call the
   existing `Field::impl_partial_env_value()`, the same per-field codegen a struct's `env_values`
   uses, and bind each non-skipped row to a positional local (`ea`, `eb`, ...). Extend
   `get_binding_idents` to return a third ident, so a field named `env` or `context` can't shadow
   the locals. Prefix the block with `let mut env = EnvManager::new(None::<&str>, #own_prefix);`.
   In Phase 0 `own_prefix` is always `None`, and the only rows are nested settings:
   `env.nested(<..>::env_values()?)?`.
2. **`impl_partial_finalize_named`**. After the defaults `MergeManager` chain, merge each env local
   over its binding with the setting's own `impl_partial_merge_into` (so a custom `merge` applies, as
   it does for a struct's env layer). Then run the existing transform and nested statements. A
   setting with an env row needs a `mut` binding, like one with a default. The skip condition grows
   to "no defaults, no env rows, no statements". Gate the env half on `#[cfg(feature = "env")]` in
   core, as `impl_partial_env_values` is.
3. **Order inside the arm:** defaults → the variant as merged from sources → env → transforms and
   nested finalization. That is the loader's order for a struct.

Sketch of the arm for `Named { #[setting(nested)] child: Child }`:

```rust
Self::Named { child: na } => {
    let mut pa = <PartialChild as PartialConfig>::default_values(context)?;   // today
    MergeManager::new(context).nested(&mut pa, na)?;                         // today
    let mut env = EnvManager::new(None::<&str>, None);                       // new
    let ea = env.nested(<PartialChild as PartialConfig>::env_values()?)?;    // new
    MergeManager::new(context).nested(&mut pa, ea)?;                         // new
    if let Some(layer) = pa { pa = Some(layer.finalize(context)?); }         // today
    Self::Named { child: pa }
}
```

As built: the env binding is `get_env_binding_ident` (`ea`, ...), next to `get_binding_idents`,
rather than a third value from it. Snapshots: `container_finalize_test::named_enum::reads_nested_env`
(new), and `supports_nested` and `container_type_test::to_tokens::named_enum` (changed). The runtime
tests are `config_test::named_variant_env`. Named variants are unreleased, so the changelog amends
their entry under Updates rather than adding a fix.

## Phase 1: `env`, `parse_env` and `env_prefix` on named variants · the feature

Mostly lifting panics, since Phase 0 already emits whatever `Field::impl_partial_env_value` returns.

1. **Field attributes.** Drop `env`, `env_prefix` and `parse_env` from
   `Field::validate_variant_args`, and keep `extend`. Every other existing check then applies
   unchanged: `parse_env` needs a key, `env` + `env_prefix` conflict, `env` on `nested` panics,
   `env_prefix` needs `nested`, and collections need `parse_env`.
2. **Container `env_prefix`.** In `validate_enum_attributes`, allow it on a `Config` enum with
   named variants, exactly as `allow_unknown_fields` is allowed today. It still panics on a
   `ConfigEnum` (whose setting reads it with `env` and `FromStr`) and on a `Config` enum without
   named variants, since nothing would derive a key there. `Field::get_env_var` already derives
   `PREFIX + NAME` from `container_args.env_prefix`, which a variant's fields share, so
   `own_prefix` in Phase 0's `EnvManager` becomes the container's prefix and nothing else changes.
   Two variants with a field of the same name share a derived key. That's fine, as only one variant
   is ever active.
3. **`#[setting(nested, env_prefix)]` inside a variant** works as-is. The variant's env block calls
   `env_values_with_prefix(Some(..))` through the same field codegen. `impl_partial_checks` iterates
   `self.inner.get_fields()`, which is empty for an enum (`container.rs:1923`), so extend the
   `HAS_ENV_PREFIX` assertion to cover named variant settings too.
4. **A parent's `#[setting(nested, env_prefix)]` on an enum stays a compile error.** A variant's env
   is read in `finalize`, which takes no prefix. The maintainer rejected a hidden
   `finalize_with_prefix` before. Collection items already can't take a parent prefix ("Cannot use
   `env_prefix` with collections or wrapped types."), so there's precedent. `HAS_ENV_PREFIX` is only
   emitted for named structs (`container.rs:1371`), so the existing G7 assertion already fires, but
   its message says to add `#[config(env_prefix)]` to the child, which is wrong for an enum that
   already has one. Reword it to cover both, something like "...a parent's prefix only applies to
   the keys a struct derives with an `env_prefix` of its own".
5. **Metadata comes for free.** A named variant's schema and `settings()` entries are built from
   each `Field`'s `impl_schema` / `impl_setting`, which already call `get_env_var_name`, so
   `SchemaField.env_var`, TypeScript `@env`, templates and api_docs pick up variant keys with no
   renderer work. Check the snapshots rather than assuming it.
6. **Tuple and unit variants are unchanged.** A tuple variant's value isn't a setting, and a nested
   one already reads its own env. `VariantArgs` has no `env`, so darling keeps rejecting it.

User-facing result:

```rust
#[derive(Config)]
#[config(env_prefix = "AUTH_")]
#[serde(tag = "type", rename_all = "lowercase")]
enum Auth {
    #[setting(default)]
    None,
    Token {
        token: String,                          // AUTH_TOKEN
    },
    Basic {
        user: String,                           // AUTH_USER
        #[setting(env = "AUTH_PASS")]
        password: String,                       // absolute, no prefix
        #[setting(nested)]
        tls: TlsConfig,                         // TlsConfig's own keys
    },
}
```

An env value fills a field **of the variant a source, or the default, picked**. `AUTH_TOKEN` does
nothing while the file says `type = "basic"`.

## Phase 2: picking the variant from the environment · needs your call

Phase 1 can fill a variant but never pick one, so an env-only deployment can't switch `Auth` from
`none` to `token`. A selector can be added without breaking Phase 1's no-double-read property:
`env_values` returns the selected variant with **every field empty**, as `default_values` does, and
the finalize arm fills it.

**Recommended shape, no new attribute:** for an internally or adjacently tagged enum with
`env_prefix`, derive the selector key from the tag, as a struct derives keys from field names:
`#[serde(tag = "type")]` + `env_prefix = "AUTH_"` → `AUTH_TYPE=token`.

- Matched exactly against each variant's serde name (`Variant::get_name`, renames applied) and
  aliases, like serde.
- Selectable variants: unit, named, and tuple variants whose values are all nested configs
  (`V(PartialChild::default())`, filled in by `finalize_with_layers`). A tuple variant holding a
  plain value, a skipped variant, or an untagged one can't be selected. An unknown value is a
  `HandlerError` listing the valid names, which `EnvManager::read` already prefixes with the
  variable name.
- Optionally, report the key as `env_var` on the tag's `Literal` field in each variant's schema.

Questions before building it:

1. **Externally tagged enums** (the default for a `Config` enum) have no tag to derive a key from.
   Should they get no selector (recommended: users add `#[serde(tag)]`), or an explicit container
   key like `#[config(env = "AUTH_TYPE")]`, which structs would then have to reject?
2. **Collections:** `finalize_with_layers` calls `env_values` for every item, so `AUTH_TYPE=token`
   would turn every item of a `Vec<Auth>` into `Token`. That's consistent with items reading env
   (`inherits_env_for_each`), but much more drastic for a selector. Should it be accepted and
   documented?
3. **Is there a concrete user?** Without one, I'd ship Phases 0 and 1 and park Phase 2 in `gaps.md`.

## Tests

- **Core snapshots** (`container_finalize_test`, `named_enum`): an explicit `env`, a derived key
  with `env_prefix`, `parse_env` with a custom `merge`, a nested setting, a nested setting with
  `env_prefix`, and a variant with defaults and env together. Read every regenerated snapshot.
- **Core panics** (`setting_env_test::named_enum`, `attribute_errors_test`): flip the three
  `errors_within_a_variant` tests into "accepts" tests. Keep the panics for `env_prefix` on a
  `ConfigEnum`, on a `Config` enum without named variants, and `extend` in a variant. Pin the
  reworded parent-prefix message.
- **Behavior** (`crates/macros/tests/config_test.rs`, a `named_enum_env` module, run with
  `--test-threads=1`):
  - the ✅ probe above, both variants;
  - precedence: default < file < env within the picked variant;
  - env for an unpicked variant is ignored;
  - root enum, bare nested, `Option<Auth>`, `Vec<Auth>` items, an enum in a tuple variant, and an
    enum in another named variant's field;
  - `append_vec` + `parse_env` applied exactly once (the `ONCE_*` tests' pattern);
  - a `required` field satisfied by env passes the finalizing `validate`;
  - a parse failure names the variable;
  - `Default::default()` reads no env.
- **Metadata** (`crates/schematic/tests/named_enum_test.rs`): `settings()` reports variant keys,
  and the TypeScript snapshot has `@env` on variant fields.
- **Builds**: `cargo build -p schematic_core` (no features), `cargo build -p schematic
  --no-default-features --features config` (env off), then `cargo test -p schematic_macros`,
  `cargo test -p schematic`, `cargo clippy --workspace --all-targets`, `cargo fmt --all --check`.

## Docs

- `book/src/config/struct/env.md`: replace the "Not supported for enums" note with an *Enums*
  section covering the variant-picked rule, shared keys, and no parent prefix (plus the selector,
  if Phase 2 lands).
- `AGENTS.md`:
  - **Named variants:** the last bullet says `env`/`env_prefix`/`parse_env` panic on a field.
  - **Runtime flow:** a named variant's finalize arm merges env over it.
  - **Decided semantics → Env keys:** a parent prefix on an enum is a compile error.
- `docs/todo/gaps.md` G7: `env_prefix` on a `Config` enum is no longer in the panic list.
- `CHANGELOG.md`: Phase 0 under fixes, Phase 1 under `#### 🚀 Updates`.

## Suggested commits

1. `fix: Read the env of nested configs in named variants.` (Phase 0)
2. `new: Support env vars in named enum variants.` (Phase 1, with docs)
3. Phase 2, only if decided.
