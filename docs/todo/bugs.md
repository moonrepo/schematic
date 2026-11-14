# Bugs

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

Every bug from the audit was fixed on `0.21-bugs`, and is listed in the `Unreleased` section of
`CHANGELOG.md`. What's left here isn't a bug, or can't be fixed without new semantics.

## Open

Nothing. B1, nested defaults and env applied once per nesting level, was fixed with a hidden
`PartialConfig::finalize_nested`, so only the outermost config applies them. See the runtime flow in
`AGENTS.md`.

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
