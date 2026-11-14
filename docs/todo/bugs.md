# Bugs

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

Every other bug from the audit was fixed on `0.21-bugs`, and is listed in the `Unreleased` section of
`CHANGELOG.md`. What's left here needs a decision, or can't be fixed without one.

## Open

**B1. Nested configs apply their defaults and env values once per nesting level** ✅ · high

- **Scenario:** a child declares `#[setting(default = vec!["d"], merge = append_vec)] list`.
  - Loading `{"child":{"list":["f"]}}` yields `["d","d","f"]`, and `["d","d","d","f"]` under a
    grandparent.
  - An env list yields `["e","e"]`.
  - With no sources at all, `load()` yields `["d","d"]` while `Parent::default()` yields `["d"]`.
  - Default handler fns also run N times.
- **Cause:** the parent's `default_values` already contains `PartialChild::default_values()`, and
  its `env_values` contains the child's env. The child's own generated `finalize` then merges both
  again.
- **Why it hides:** idempotent (replace) merges don't show it. Tests use `append_vec` only at the top
  level.
- **Where:** `crates/core/src/container.rs:1307-1340`, `crates/core/src/field_value.rs:75`.
- **Fix (codegen only, no API):** in the generated `finalize`, blank the bare-nested fields of the
  default and env layers before merging. Keep `Some(Default::default())` so the child is still
  finalized. A field with a parent `env_prefix` still needs its parent-prefixed read kept.

- **Why the proposed fix doesn't work:** blanking the nested fields of the default layer, as
  `Some(Default::default())`, is only safe for a struct, whose empty partial is a no-op to merge. An
  enum's partial has no empty state, so its `Default` is the default variant holding the type
  default, which would override a variant default like `#[setting(default = true)] Enabled(bool)`.
  The derive can't tell a nested struct from a nested enum, so it can't blank only structs.
- **Fix that works for both:** a hidden `PartialConfig::finalize_nested`, which by default calls
  `finalize`. The derived struct implements it as `finalize` without merging its own defaults or
  env, and a parent calls it for each nested setting. Each default and env value is then applied
  once, by the root, and the own-before-parent env order is unchanged. This changes when work
  happens in `finalize`, and adds a hidden trait method, so it's waiting on a decision.
- **Also affected:** the fix for a default enum variant holding a nested config (now in its
  `default_values`) duplicates that config's defaults the same way, when it's finalized.
- **Remaining with the current mechanism, even once B1 is fixed for defaults:** a nested config with
  a parent `env_prefix` reads its own prefix in both the parent's env layer and its own `finalize`,
  so a non-idempotent merge applies it twice. Fixing it needs the parent to read only its own
  prefix, which changes `EnvManager` from own-first to parent-only when a parent prefix is given.

## Won't fix, or not a bug

- **An enum variant merge returning `None` becomes `unwrap_or_default()`** 📖 · low
  - On a struct, `None` clears the setting; here it yields the variant with a default inner value.
  - An enum's `merge` is `&mut self`, so it has to leave some variant behind, and the `Option` that
    `None` would clear belongs to the parent. Supporting it would need new merge semantics.
- **`chrono::Days`, `Months`, `IsoWeek`, `NaiveWeek` and `Date<Tz>` have no serde impl, so their
  schemas describe nothing serde accepts** 📖 · low
  - Removing the impls would break a config that holds one in a `#[serde(skip)]` field, as those are
    still inferred (C9). Revisit alongside C9.
- **`Box<Path>` still can't be a setting** · low
  - The partial now keeps the wrapper, but std has no `Default` for `Box<Path>`, which every setting
    needs for `from_partial`. `Box<OsStr>` and `Box<CStr>` have one, but no `Schematic` impl.
- **An indirect cycle through a `Box`, like `A { b: Option<Box<B>> }` and `B { a: Option<A> }`, still
  gives the partial an infinite size** · low
  - The `Box` is only kept for a config that holds its own type, as keeping it for every nested
    config would change the type of every boxed partial field. A collection, like `Vec<B>`, works.
