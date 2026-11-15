# Gaps

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

**G1. An in-memory or CLI layer** (#148) · fixed on `0.21-gaps`

`ConfigLoader::partial(name, T::Partial)` adds a layer recorded as `Source::Memory { name }`. It's
merged in declaration order, validated with its name as the location, and may extend other sources;
a relative file resolves against the working directory, as a memory source has no location of its
own. `ConfigLoader::source` rejects a `Source::Memory`, which has no partial to load. Like every
layer it sits below env.

`ConfigLoader::overrides`, a layer above env, was not built, by decision: it would either skip
transforms on override values or go through the hidden `finalize_nested`. CLI-over-env is left to
the argument parser's own env support, which the book now says.

**G2. Merge helpers that real configs need** · fixed on `0.21-gaps`

Added to `merge`: `merge_partials` (a map of nested configs, deep merged by key through a
`MergeableMap` trait implemented for `HashMap` with any hasher, `BTreeMap`, and `IndexMap`),
`append_vec_unique` (moon's dedupe-aware append, keeping duplicates already in `prev`),
`merge_iter` (any `Extend`), and `merge_indexmap`/`merge_indexset` under `type_indexmap`, which now
enables `indexmap`. `merge_hashmap`/`merge_hashset` take any `S: BuildHasher`, and the `BTree*`
merges only need `Ord`. moon's `merge_iter` and `merge_partials` can be replaced by these.

`merge_partials` hands the items the parent's context, as every `merge` function receives it, so it
can't be combined with `#[setting(nested, context = func)]`. A hasher parameter means a direct call
like `merge_hashmap(HashMap::from_iter(..), ..)` no longer infers `RandomState`.

**G3. Config files choosing their own merge strategy** (#175) · partly fixed on `0.21-gaps`

`MergeList<T>`, merged with `merge::merge_list`. A source writes `[..]` to replace the previous items, or a
map with any of `prepend`, `append`, and `replace`. It holds the items and the index the previous
items go at (`None` when it replaces them), which makes merging associative. That matters, because
layers merge into each other before the result is merged over the defaults. It derefs to `[T]`,
implements garde's `HasSimpleLength` so the length validators accept it, (de)serializes both forms
with a hand-written visitor (so an unknown key names the expected ones), and its schema is a union
of the list and a closed struct of optional lists.

The derive infers `merge = merge_list` for a setting or variant value whose outermost type, within
an `Option`, is named `MergeList` (`Value::is_merge_list`), the way it knows layers by name, chosen
over autoref specialization, which would change every setting's generated merge. An explicit
`merge` wins, a collection of them is replaced, and an alias isn't known. `nested` on one panics.

Not built:

- Nested items (`MergeList<Child>`), the issue's own case. Core would have to treat `MergeList` as
  a collection layer, so its items are partials, finalized and validated. Ask first.
- Env vars: there's no `parse_env` for it.

**G4. Provenance: which layer set a value** · fixed on `0.21-gaps`

The loader drives the layers itself: `default_values` as a `Source::Defaults` layer below every
source, `env_values` as a `Source::EnvVars` layer above them, merged in order, then `finalize`.
`ConfigLoadResult.layers` reports exactly those, each only when it sets something. By decision this
was a breaking change rather than an inspection-only copy, which would have called each default
handler twice and could report a default other than the one used:

- `PartialConfig::finalize` only runs transforms and nested finalization, taking over from the
  removed hidden `finalize_nested`.
- Collection items and variant values, which no layer holds the defaults or env of, go through the
  hidden `internal::finalize_with_layers`.

It also fixed a root enum's default variant value never being applied when loading.

Not built: source-aware transforms (`&Source` in the transform context), and resolving relative
paths against the declaring file, which moon does after loading.

**G5. Schema metadata that never reaches the schema** · partly fixed on `0.21-gaps`

Fixed, mostly in core so that every renderer agrees:

- **`Option` fields** are `optional`, as serde fills in an omitted one, which takes them out of
  JSON Schema's `required` and gives them `?` in TypeScript (and an Optional tag in api_docs).
- **`required` on `Option`** is described by the type it holds, so it's neither optional nor
  nullable. That covers what `SchemaField.required` was proposed for, without a new field.
- **Unknown fields:** `StructType.allow_unknown_fields`, inverted from the proposed
  `deny_unknown_fields` so that hand-written, std, and synthesized variant schemas stay closed by
  default. The derive opens a config with `allow_unknown_fields` and a `Schematic` type without
  `deny_unknown_fields`; JSON Schema then leaves out `additionalProperties`, also through flatten.
- **`#[serde(transparent)]`** on a `Schematic` named struct uses its field's schema (a tuple newtype
  already did). On a `Config` it panics, as the partial isn't transparent and serde won't make it
  so, with every setting defaulted.
- **Integers** render as `"type": "integer"` in JSON Schema, with their kind's range up to 32 bits;
  an explicit bound narrows it.

Still open:

- **Validator constraints:** they never reach the schema. Built-ins with literal args (`min_length`,
  `in_range`, `regex`, `email`, `url`, `ip_v4`, `uuid`) could set `StringType`/`IntegerType` fields
  at derive time, following the `path_names_a_value` precedent. JSON, Pkl and api_docs already render
  these constraints. 📖
- **Other missing metadata:** 📖
  - Aliases are not in `required` alternatives, and enum variant aliases never reach `EnumType`.
  - `read_only` and `write_only` are never set.
  - JSON Schema leaves doc `@tags` in the description.
  - There is no `examples` support (an `@example` tag would be a natural source).
  - The deprecation message is dropped in JSON Schema.
  - Unit-enum variant descriptions are dropped.
- api_docs and Pkl don't reflect `allow_unknown_fields`.

**G6. Templates drop flattened fields** · fixed on `0.21-gaps`

Every template renderer now inlines a flattened struct's fields in place of the field that holds it,
through `template::inline_flattened_fields`, resolving references and an `Option`, recursively. The
struct's own field of a name takes precedence. TOML was worse than dropping them: `extract_sections`
didn't skip flattened fields, so a flattened struct became a `[table]` of its own, which the
partial's `deny_unknown_fields` rejected on load. A flattened map still renders nothing.
`renderer_fixes_test.rs` (`flattened_templates`) loads each format's template back through the
loader and checks its layer set every flattened setting.

Found while testing it: a config can't load a flattened struct that flattens another, as serde
doesn't support `deny_unknown_fields` with `flatten`. One level works, as a plain flattened
struct takes its keys from the parent's buffer, while one with a flattened field of its own reads
them as a map without taking them, so the parent rejects them as unknown. Every setting of the
middle struct fails, not only the innermost. `#[config(allow_unknown_fields)]` on each is the
workaround. Fixing it means the derive dropping `deny_unknown_fields` somewhere, a strictness
trade-off to decide.

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
