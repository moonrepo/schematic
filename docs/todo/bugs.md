# Bugs

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

## Loading, merging, env

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

**B2. Extends cycles overflow the stack, and diamonds load twice** ✅ · high

- **Cycle:** `a.json` extends `b.json`, which extends `a.json`. The process aborts with "thread
  'main' has overflowed its stack".
- **Diamond:** `d1 → [d2, d3]`, where both extend `d4`. The layers are `[d4, d2, d4, d3, d1]`, so
  `d4`'s list items are appended twice.
- **Where:** `crates/schematic/src/config/loader.rs:193-232`, `:301-304`. There is no visited set.
- **Fix:** thread a stack of canonical paths and URLs through `parse_into_layers`, and add
  `ConfigError::ExtendsCycle(chain)`. Decide whether a diamond loads a shared base once (probably
  yes). Small, with one new error variant.

**B3. RON cannot read ordinary RON** ✅ · high (for RON users)

- **Scenario:** `(name: "x", port: 1)` fails with "Expected option". Every partial field is an
  `Option`, and RON requires `Some(..)` unless `implicit_some` is enabled. There are no RON tests.
- **Where:** `crates/schematic/src/config/formats/ron.rs:23`.
- **Fix:** deserialize with `Options::default().with_default_extension(Extensions::IMPLICIT_SOME)`,
  and add fixtures. A few lines.

**B4. `EnvManager` reads the parent prefix first, contradicting its own doc and `AGENTS.md`** ✅ · med

- **Cause:** `EnvManager::new(override_prefix, prefix)` builds `vec![override_prefix, prefix]`, and
  the first hit wins. The codegen passes the parent's prefix as `override_prefix`. The comment
  directly above the field says "The container's own prefix comes first".
- **Effect:** for scalars, `finalize`'s unprefixed re-read masks this. For an `append_vec` list with
  both variables set, the result is `["parent","own"]`.
- **Where:** `crates/schematic/src/internal/env.rs:7-25`, `crates/core/src/container.rs:1244`.
- **Fix:** swap the order to `vec![prefix, override_prefix]`. This is a one-liner that matches the
  decided semantics.

**B5. Variant `rename_all` disagrees with serde for consecutive capitals** ✅ · med-high

- **Scenario:** `#[serde(rename_all = "kebab-case")] enum Platform { MacOS, HTTPServer }`.
  - serde accepts `mac-o-s` and `h-t-t-p-server`.
  - `ConfigEnum` `Display`/`FromStr`, schemas and literal-union names use `mac-os` and
    `http-server`.
  - So env vars, files and schemas disagree. camelCase and SCREAMING have the same problem. Field
    conversion agrees with serde.
- **Cause:** `convert_case` treats acronyms as one word, while serde's `RenameRule` splits before
  every uppercase letter.
- **Where:** `crates/core/src/utils.rs:50-74`.
- **Fix:** port serde's `RenameRule::apply_to_field/apply_to_variant` (about 40 lines). This also
  lets `config` drop `convert_case`.

**B6. Raw identifiers leak `r#` into every derived name** ✅ · med-high

- **Scenario:** `pub r#type: String` with `env_prefix = "APP_"`.
  - The `settings()` key, schema field and validation path are all `r#type`.
  - The env key is `APP_R#TYPE`, so `APP_TYPE` is never read.
  - `type`, `ref` and `override` are common config keys.
- **Where:** `crates/core/src/field.rs:169`, `:226`, plus the variant and container equivalents.
- **Fix:** `syn::ext::IdentExt::unraw()`. Trivial.

**B7. `is_secure_url` can be bypassed with a substring** ✅ · med (security-adjacent)

- **Scenario:** each of the following counts as "secure", so plain-HTTP config is fetched or accepted
  by `validate::url_secure` and `extends_string`:
  - `http://evil.example/c.yml?h=127.0.0.1`
  - `http://localhost.evil.example/…`
  - `http://127.0.0.1.nip.io/…`

  `HTTPS://` (uppercase) is rejected.
- **Where:** `crates/schematic/src/helpers.rs:20-26`.
- **Fix:** parse the scheme and host. Allow `http` only for the exact hosts `localhost` or
  `127.0.0.1`, and match `https` case-insensitively. Small.

Smaller runtime bugs:

- **JSON whitespace or comment-only files fail with EOF, while empty files pass** ✅ · low
  - The empty check runs before trimming and comment stripping.
  - Where: `formats/json.rs:25`.
- **`is_url_like` treats any value starting with `www` as a URL** ✅ · low
  - `extends: "www/base.json"` fails with "only secure URLs".
  - Where: `helpers.rs:14-16`.
  - Fix: require `www.`.
- **`EnvManager.count` is a `u8`** 📖 · low
  - More than 255 env hits panics in debug and wraps to 0 in release, which silently drops every env
    value.
  - Where: `internal/env.rs:6`.
  - Fix: use a `bool`.
- **An enum variant merge returning `None` becomes `unwrap_or_default()`** 📖 · low
  - On a struct, `None` clears the setting; here it yields the variant with a default inner value.
- **`validate::in_range` is documented as non-inclusive but is inclusive** ✅ · low
  - Where: `validate/number.rs:5`.

## Derive codegen

**B20. Collections of nested configs are silently emptied unless the type name ends in
`Vec`/`Set`/`Map`** ✅ · high

- **Scenario:** `#[setting(nested)] items: VecDeque<Item>` loaded from `{"items":[{..},{..}]}`
  yields `items: []`.
  - The same happens to `LinkedList`, `BinaryHeap`, and any user generic.
  - A non-nested `VecDeque<Arc<String>>` loses data the same way, because wrapper stripping forces
    the mapping.
- **Cause:** `extract_layer` classifies by name suffix. Anything else becomes `Layer::Unknown`, which
  `from_partial` and `finalize` emit as `#collection::default()`.
- **Where:** `crates/core/src/value.rs:779-803`, `:320-324`, `:416-420`.
- **Fix:**
  - Minimum: panic at derive time on `Layer::Unknown` where a mapping is required.
  - Better: map every collection layer with `.into_iter().map(..).collect()`, as in C2.

**B8. `Config::settings()` overflows the stack for any recursive config** ✅ · high

- **Scenario:** `struct Tree { #[setting(nested)] children: Vec<Tree> }` loads fine, but calling
  `Tree::settings()` aborts the process.
- **Cause:** `.nested(Child::settings())` is built eagerly.
- **Where:** `crates/core/src/container.rs:448`.
- **Fix:**
  - Simplest: skip `.nested(..)` for `Self`. This misses A→B→A cycles.
  - Robust: a thread-local stack in a tiny `internal::nested_settings::<T>()` helper.

**B9. `Option<Box<Self>>` recursion can't be expressed in a `Config`** ✅ · med

- **Scenario:** a `Config` with `#[setting(nested)] next: Option<Box<Self>>` fails to compile.
  - The partial strips the `Box`, which gives E0391 (infinite size).
  - `Self` is also emitted inside the partial, where it means the partial, and `PartialSelf` gets
    generated.
- **Fix:** keep `Box` when the inner type is the container itself, and substitute `Self` at derive
  time (part of C1).

**B10. A field-level `#[serde(default = "fn")]` makes the field optional in the schema but is never
called** ✅ · med

- **Scenario:** `#[serde(default = "default_port")] port: u16` loaded from `{}` yields `port: 0`.
- **Where:** `crates/core/src/field.rs:302-307` counts it as optional, but
  `crates/core/src/field_value.rs:90-109` only reads `#[setting(default)]`.
- **Fix:** treat it as `#[setting(default = default_port())]`, or panic and point to
  `#[setting(default)]`. Trivial.

**B11. `#[serde(with | deserialize_with | serialize_with)]` is dropped from the partial** ✅ · med

- **Scenario:** `#[serde(with = "humantime_serde")] timeout: Duration` loaded from `"5s"` fails,
  because the partial uses `Duration`'s own `{secs, nanos}` encoding.
- **Where:** these keys aren't parsed (`args.rs:113-127`, `allow_unknown_fields`).
- **Fix:**
  - Simplest, consistent with loud failure: panic with "use
    `#[setting(partial(serde(with = …)))]` with an `Option`-aware module", and document it.
  - Alternative: generate an `Option` adapter, which costs two fns per field.

**B21. `Default` of an enum whose default variant holds a nested config skips that config's
defaults** ✅ · med

- **Scenario:** `#[setting(default, nested)] File(FileSource)`, where `FileSource.path` has a
  default.
  - Loading `{}` gives the default path.
  - `Source::default()` gives `""`.
- **Where:** `crates/core/src/variant.rs:659-687`.
- **Fix:** for a bare nested single-value variant, emit the child's `default_values(..)`.

Smaller derive bugs:

- **`required` on a variant holding values of different types doesn't compile** ✅ · med
  - It emits `[pa, pb].iter().any(..)`, an array of heterogeneous types.
  - Where: `variant.rs:1006`.
  - Fix: emit `pa.is_none() || pb.is_none()`.
- **The validation path for a flattened nested config names a key the user never wrote**
  (`common.label` instead of `label`) ✅ · med
  - Where: `field.rs:662-668`.
  - Fix: validate at the current path.
- **Unsized types other than `str`, `[T]` and `dyn` get their wrapper stripped** ✅ · med
  - `Box<Path>`, `Arc<Path>`, `Box<OsStr>` and `Box<CStr>` put the unsized type into the partial (20
    errors).
  - Where: `value.rs:633-643`.
  - Fix: add these types to the unsized list.
- **String-literal defaults convert through the innermost type's `TryFrom<&str>`** ✅ · med-low
  - `default = "abc"` on `Box<str>` becomes `str::try_from`.
  - `default = "127.0.0.1"` on `Option<IpAddr>` fails (`IpAddr` is `FromStr`-only).
  - Generic inner types emit invalid syntax.
  - Where: `value.rs:201-205`.
  - Fix: use `<#ty as TryFrom<&str>>` at the right layer, optionally falling back to `FromStr`.
- **The partial's `schema_name` ignores a container `rename`** ✅ · low
  - `PartialChild` vs. the parent's reference to `PartialKid`.
  - Where: `container.rs:914`.
  - Fix: use `format!("Partial{}", self.get_name())`.
- **Fully qualified `std::sync::Arc` / `std::collections::BTreeMap` paths fail without a `use`** ✅ ·
  med
  - `Arc::new`/`Rc::new` and bare collection idents are emitted unqualified.
  - Where: `value.rs:24`, `:272`, `:356`, plus about nine more.
  - Fixed by C2.

## Schemas and renderers

**B12. A suffixed integer default doesn't compile with `schema` on** ✅ · high

- **Scenario:** `#[setting(default = 5u8)]` emits `LiteralValue::Uint`, but the variant is `UInt`.
  `5i32` produces "expected `isize`, found `i32`".
- **Where:** `crates/core/src/utils.rs:222-227`.
- **Fix:** rename to `UInt` and emit the unsuffixed `base10_digits()`. Trivial.

**B13. `Box`/`Rc`/`Arc`/`Cow`/`&T`/`&mut T` don't forward `schema_name()`** ✅ · high

- **Scenario:** a recursive `#[derive(Schematic)] struct Linked { next: Option<Box<Linked>> }`
  overflows the stack in `SchemaGenerator::add`.
  - `a: Box<Shared>` is inlined anonymously, while `b: Shared` gets a `$ref`.
  - A `#[setting(nested)] Arc<Child>` gets no reference and no `Partial` rename.
- **Where:** `crates/types/src/lib.rs:90-124`.
- **Fix:** add `fn schema_name() -> Option<String> { T::schema_name() }` to each of the six impls.

**B14. JSON Schema: a `$ref` to the root type dangles** ✅ · high

- **Scenario:** `TreeConfig { #[setting(nested)] children: Vec<TreeConfig> }` renders
  `"$ref": "#/definitions/TreeConfig"`, but the root is inlined and never added to `definitions`.
  A child pointing back at its parent breaks the same way.
- **Where:** `crates/schematic/src/schema/renderers/json_schema.rs:676-700`.
- **Fix:** render a root reference as `"#"`, or also emit the root under `definitions`.

**B15. JSON Schema: a flattened struct becomes `additionalProperties`** ✅ · high

- **Scenario:** `#[setting(flatten, nested)] base: BaseConfig` gives `additionalProperties: {Base
  schema}`. A valid `{"own": true, "shared": "x"}` is then rejected, because `"x"` isn't an object.
- **Where:** `json_schema.rs:544-556`.
- **Fix:** resolve the struct (it may be a reference) and merge its `properties` and `required` into
  the parent. Keep `additionalProperties` for flattened maps only.

**B16. Internally tagged variants holding a named type lose the tag, and can pollute the shared
definition** ✅ · high

- **Scenario:** `#[serde(tag = "type")] enum Tagged { One(Shared), Two(Shared) }`.
  - JSON renders `anyOf [$ref Shared, $ref Shared]`, and TS renders `Shared | Shared`.
  - serde's real `{"type":"One","a":"x"}` fails validation.
  - If `Tagged` registers first, the `Shared` definition itself requires `"type": "One"`
    everywhere.
  - If `Shared` is on the name stack, `add_field` on the `Reference` silently drops the tag.
- **Where:** `crates/core/src/variant.rs:573-579`. Pkl already works around it
  (`pkl_schema.rs:1186-1203`).
- **Fix:** clear `item.name` after adding the tag, which makes it an anonymous struct. Then simplify
  Pkl's workaround.

**B17. TypeScript: names that aren't valid identifiers are emitted unquoted** ✅ · high

- **Scenario:** `rename_all = "kebab-case"` gives `dash-name: string;`, and `EnumFormat::ValuedEnum`
  gives `very-loud = 'very-loud',`. Both are syntax errors.
  - This was latent while the old derive applied camelCase; kebab-case is now the usual choice.
- **Where:** `typescript.rs:473`, `:274-283`, `:299-307`.
- **Fix:** quote keys that don't match `^[A-Za-z_$][\w$]*$`, quote enum member names, and escape `'`
  and `\` in literals.

**B18. A field's default leaks from its type's schema** ✅ · med

- **`Option<Mode>`:** a field with no default reports `default: "foo"` (the enum's default variant)
  in JSON, TS, YAML and api_docs. The real default is `null`.
- **Shared types:** `#[setting(default = "bar")] first: Mode` mutates the shared named schema, so
  the Pkl alias and the api_docs enum page depend on registration order.
- **Where:** `crates/types/src/schema_type.rs:51-58`, `crates/core/src/field.rs:470-474`.
- **Fix:** add `SchemaField.default: Option<LiteralValue>`, filled from literal field defaults.
  Renderers read it and fall back to the type for hand-written impls. Medium, because every renderer
  is touched.

**B19. Templates emit invalid TOML, YAML and JSONC** ✅ · med

- **TOML `Option<Struct>`:** `#[setting(nested)] maybe: Option<BaseConfig>` renders
  `maybe = count = 0`.
- **TOML `null`:** `serde_json::Value` renders `value = null`, but TOML has no null.
- **YAML:** an adjacently tagged enum renders `adj: kind: "Unit"`.
- **String escaping:** strings are not escaped. `"it's \"quoted\""` breaks all three formats.
- **JSONC trailing comma:** the last-comma index counts hidden and flattened fields, which leaves a
  trailing comma.
- **JSONC empty struct:** a nested empty struct panics with "attempt to subtract with overflow".
- **Where:** `toml_template.rs:37-90`, `template.rs:76`, `:328`, `:387`, `jsonc_template.rs:115`,
  `:134`.
- **Fix:** small per item. Reuse `is_nested_type` for TOML sections, comment out nulls in TOML,
  escape strings per format, and compute commas after filtering.

Smaller schema bugs:

- **JSON Schema maps with non-string keys can never validate** ✅ · med
  - `HashMap<u32, _>` gives `propertyNames: {type: number}`, but JSON keys are strings.
  - Where: `json_schema.rs:489`.
- **JSON Schema `Unknown` excludes `null`** ✅ · med
  - A `serde_json::Value` set to `null` is rejected.
  - Where: `json_schema.rs:659-673`.
  - Fix: emit `{}`.
- **TypeScript flatten creates a duplicate identifier when the field is named `base`** ✅ · med
  - Both get the name `{Name}Base`.
  - Where: `typescript.rs:190-224`.
- **`skip_deserializing` fields and variants are still in schemas** ✅ · med
  - Such a field is listed as *required*, and such a variant is in the TS union, although serde
    rejects both.
  - Where: `field.rs:520-531`, `variant.rs:451`, `container.rs:1003-1006`.
  - Fix: treat them like `skip`, or set `read_only`.
- **`definitions_path` only changes the `$ref` prefix** 📖 · low
  - The container key stays `definitions`, so `"#/$defs/"` dangles.
  - Tuples always use draft-07 `items: [..]`.
  - `inline_subschemas` is never read.
  - Where: `json_schema.rs:611`, `:705`.
- **`lit_to_value` unwraps on NaN/∞** 📖 · low
  - Only reachable from hand-written schemas.
  - Where: `json_schema.rs:144-145`.

**`externals.rs` impls that don't match their serde encoding** ✅ · med:

- `chrono::Duration` (TimeDelta) is modelled as a duration string, but serde emits `[secs, nanos]`.
- `NaiveDateTime` and `NaiveTime` claim `date-time` and `time`. Those formats require an offset,
  while serde emits `"2024-01-02T03:04:05"`.
- `rust_decimal::Decimal` is `string` only, but serde also accepts numbers, so the schema should be
  `string | number`.
- `Days`, `Months`, `IsoWeek`, `NaiveWeek` and `Date<Tz>` have no serde impl at all.
- `IndexSet` lacks `unique`, which `HashSet` and `BTreeSet` set.
- `serde_json::Number` is typed `I64`.
