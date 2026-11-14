# Gaps

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**G1. An in-memory or CLI layer** (#148) 📖 · med

- **Today:** there are two workarounds.
  - Serialize a clap partial to JSON and call `.code()`. This needs the `json` feature.
  - The issue's recipe: `load_partial → finalize → merge → from_partial`. This silently skips final
    validation.
- **Simplest addition:** `ConfigLoader::partial(name, T::Partial)` backed by a
  `Source::Memory { name }`. It sits below env, consistent with not changing `finalize`.
- **Optional:** `ConfigLoader::overrides(T::Partial)`, merged after `finalize` and before
  validation. This gives CLI precedence over env without touching `finalize`. Caveat: transforms
  don't run on override values.

**G2. Merge helpers that real configs need** 📖 · med

- **What moon hand-rolls:** moon's `crates/config/src/merging.rs` re-implements:
  - `merge_iter`, an append for any `Extend` collection;
  - `merge_partials`, a deep merge of `Map<K, V: PartialConfig>` by key;
  - a dedupe-aware append.
- **Where the built-ins fall short:**
  - `merge_hashmap` and `merge_hashset` only accept the default hasher, so `FxHashMap` can't use
    them.
  - `merge_btreemap` and `merge_btreeset` require a `Hash` bound they don't need.
  - There are no `IndexMap`/`IndexSet` merges under `type_indexmap`.
- **Fix:** generic `S: BuildHasher` versions, `append_unique`, `merge_iter`, `merge_partials` (it
  fits today's merge signature through `&V::Context`), and indexmap variants. All additive and cheap.

**G3. Config files choosing their own merge strategy** (#175) 📖 · med

- **What blocks it:**
  - Merge fns see only `(prev, next, &Ctx)`, so the strategy has to travel inside the value.
  - Variant merges only run when prev and next are the same variant; otherwise next replaces prev
    (`container.rs:1432`, `variant.rs:869`). That rules out the issue's enum-wrapper design.
  - moon has the same need: its own `MergeStrategy { Append, Prepend, Preserve, Replace }`.
- **Simplest option:** a runtime newtype next to `RegexSetting`. `MergeList<T>` deserializes `[..]`
  or `{ append | prepend | replace: [..] }`, derefs to `[T]`, and pairs with a
  `merge::by_directive`. No derive change for scalar items.
- **Cost:** nested items (`MergeList<Child>`) would need core to treat it as a collection layer,
  which is medium. Ask before building that part.

**G4. Provenance: which layer set a value** ✅ · med

- **Today:** defaults and env never appear in `ConfigLoadResult.layers`, and transforms get no
  source. A value that came from env looks like it came from the only file.
- **Related:** moon post-processes loaded configs to resolve relative paths against the declaring
  file (`config-loader/src/config_loader.rs:180-186`), because schematic can't.
- **Cheap first step:** add `Source::Defaults` and `Source::Env` layers to the result, computed by
  the loader for inspection only (`finalize` unchanged). Source-aware transforms (`&Source` in the
  transform context) are the bigger follow-up.

**G5. Schema metadata that never reaches the schema** · med

- **`required` on `Option`:** `#[setting(required)]` on an `Option` is reported as nullable or
  optional everywhere (a known follow-up). Fix: add `SchemaField.required`. ✅
- **`Option` fields in JSON Schema and TS:** they are marked `required`, although serde accepts
  omission. api_docs and Pkl already treat them as optional, and the book says otherwise
  (`field.rs:302`, `json_schema.rs:561`). ✅
- **`additionalProperties: false`:** it is unconditional, even with
  `#[config(allow_unknown_fields)]` or a plain serde struct. Fix: add
  `StructType.deny_unknown_fields`. ✅
- **`#[serde(transparent)]`:** it is not parsed, so a wrapper renders `{inner}` instead of the inner
  type. ✅
- **Validator constraints:** they never reach the schema. Built-ins with literal args (`min_length`,
  `in_range`, `regex`, `email`, `url`, `ip_v4`, `uuid`) could set `StringType`/`IntegerType` fields
  at derive time, following the `path_names_a_value` precedent. JSON, Pkl and api_docs already render
  these constraints. 📖
- **Integer types in JSON Schema:** they render as `"type": "number"` with no range, so `1.5` and
  `-3` pass for a `u8`. Pkl already maps the kinds. ✅
- **Other missing metadata:** 📖
  - Aliases are not in `required` alternatives, and enum variant aliases never reach `EnumType`.
  - `read_only` and `write_only` are never set.
  - JSON Schema leaves doc `@tags` in the description.
  - There is no `examples` support (an `@example` tag would be a natural source).
  - The deprecation message is dropped in JSON Schema.
  - Unit-enum variant descriptions are dropped.

**G6. Templates drop flattened fields** ✅ · med

- Every template renderer `continue`s past `flatten`, so `ExtendingConfig`'s YAML template is just
  `own: false`.
- Fix: inline the flattened struct's fields, resolving references.

**G7. Attributes that parse and do nothing** · med

These complement the known `skip_deserializing_if`. Per the loud-failure policy, each should panic:

- `#[config(before_parse)]` on a `Config`.
- `#[variant(fallback)]` on a `Config` enum.
- `env_prefix` and `allow_unknown_fields` on enum containers. A parent's `nested, env_prefix` on an
  enum is a no-op.
- `#[default]` on a `Config` enum variant is copied to the partial and can never compile. ✅
- `tag` or `untagged` on a unit-only enum is forwarded to the partial, while the schema still says
  strings.

**G8. Loader capabilities** 📖 · low-med

- **URLs:**
  - `ureq` runs with no timeouts, so a hung server hangs loading.
  - There is no way to add auth headers.
  - `MemoryCache` never expires.
  - There is no shipped file cacher.
- **Formats:**
  - The format comes from the extension only, and a `#fragment` breaks it.
  - There is no explicit format override for extensionless files or URLs.
- **Extends:**
  - Extending from a code source is always rejected, even when the source has an absolute virtual
    path.
  - Relative extends from a URL parent are unsupported.
- **`$schema`:** a top-level `"$schema"` key (which editors want) is rejected by
  `deny_unknown_fields`. Fix: strip it in the JSON and YAML formats, or document the
  `#[serde(rename = "$schema")]` field pattern.

**G9. Validator limits** 📖 · low

- A validator returns one error, so `items[1]` is reported but `items[2]` isn't, and a garde
  `Report` can't be mapped.
- `validate::regex(..)` compiles, and panics on a bad pattern, on every validation pass.
- There are no one-sided `min` or `max` validators.
- There is no container-level `#[config(validate = fn)]`.
- Per-layer validation runs before transforms, so a "must be absolute" validator fails on raw
  layers. The `finalizing` flag exists for this, but it is undocumented.
