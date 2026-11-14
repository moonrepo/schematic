# Audit TODO — 2026-11-14

An audit of `master` at `5be067c` (schematic 0.20.12, core 0.1.11). It covers the derive
(`crates/core`, `crates/macros`), the runtime (`crates/schematic/src/config`, `internal`, `merge`,
`env`, `validate`), and schemas (`crates/types`, `crates/schematic/src/schema`). It is also informed
by the open issues and by how moon uses the library.

Goals: **fix bugs and fill gaps**, **improve devex**, **improve composability**.

Nothing here argues against the "Decided semantics" in `AGENTS.md`. The one place the code
*contradicts* a decided semantic is called out as a bug (B4). Proposals follow the preference for a
small derive surface. Each one says what it costs, and the simplest option comes first.

**Legend.** ✅ reproduced by compiling or running a probe crate. 📖 found by reading only.
Severity is **high**, **med** or **low**.

The probe crates were throwaway. Each ✅ item gives enough of a scenario to rebuild its repro.

## Files

| File | IDs | Contents |
| --- | --- | --- |
| [bugs.md](bugs.md) | `B1`–`B21` | Wrong behavior: loading, merging and env; derive codegen; schemas and renderers |
| [gaps.md](gaps.md) | `G1`–`G9` | Missing capabilities |
| [devex.md](devex.md) | `X1`–`X11` | Error messages, defaults, papercuts |
| [composability.md](composability.md) | `C1`–`C10` | Nesting, contexts, hand-written impls, custom renderers, features |
| [docs.md](docs.md) | — | Places the book, README or `AGENTS.md` disagree with the code |

## Top priorities

Ordered by user impact per unit of work.

1. **Fix nested defaults and env being applied once per nesting level (B1).** Non-idempotent merges
   such as `append_vec` duplicate default values.
2. **Fix extends cycles aborting the process (B2).** A typo in `extends` kills the CLI with a stack
   overflow.
3. **Fix collections of nested configs silently emptied (B20).** `VecDeque<Child>` and similar types
   load as empty.
4. **Resolve nested types by full path (C1).** This unblocks nesting a config from another module,
   generic nested configs, and `Self`.
5. **Fix recursion end to end (B8, B9, B13, B14).** `Box`/`Rc`/`Arc` lose `schema_name`, `settings()`
   overflows, and the JSON Schema root `$ref` dangles.
6. **Fix variant `rename_all` casing disagreeing with serde (B5).** `MacOS` ↔ `mac-o-s`.
7. **Turn derive errors into spanned compile errors (X1).** Today a typo in an attribute prints a raw
   `Debug` dump of the darling error.
8. **Make loader errors name the right place (X3–X6).** TOML syntax errors have no file, YAML type
   errors have no span, and validation is blamed on the last file.
9. **Close the schema fidelity gaps that make generated schemas reject valid config (B14–B17).**
   Flattened structs, the internal-tag `$ref`, non-string map keys, `Unknown` excluding `null`, and
   TypeScript keys that aren't valid identifiers.
10. **Ship the small loader and merge additions users already hand-roll (G1–G3).** A CLI or
    in-memory layer, the merge helpers moon wrote, and a runtime merge strategy (#175).

## Quick wins

Each item is trivial or a few lines, and every one is confirmed except where marked:

1. Forward `schema_name` in `Box`/`Rc`/`Arc`/`Cow`/`&`/`&mut` impls (B13).
2. Fix `LiteralValue::Uint` → `UInt` and unsuffix the integer (B12).
3. Swap the `EnvManager` prefix order (B4).
4. Change `EnvManager.count` to a `bool` 📖.
5. `unraw()` identifiers (B6).
6. Enable RON `IMPLICIT_SOME` (B3).
7. Map TOML syntax errors to `ParserError` (X3).
8. Parse the host in `is_secure_url` (B7).
9. Require `www.` in `is_url_like`.
10. Report darling errors with `write_errors()` (X1).
11. Emit `||` for `required` on multi-value variants.
12. Validate a flattened nested config at the current path.
13. Add `Path`/`OsStr`/`CStr` to the unsized types.
14. Use `get_name()` in the partial's `schema_name`.
15. Make JSON `Unknown` → `{}`.
16. Fix the JSONC trailing comma and empty-struct panic.
17. Treat JSON whitespace or comment-only files as empty.
18. Fall back to `type_name` for the loader name (X6).
19. Add `Debug` for `ConfigLoadResult`.
20. Skip schema inference for `skip` fields (C9).
21. Fix the merge built-in bounds and add generic hashers (G2).
22. Fix the `in_range` doc and the book feature names.

## Larger changes worth sequencing together

- **C1 + C2 + B20 + B9:** keep the full nested `syn::Type`, emit qualified paths, and use one
  `collect()`-based layer walker. This fixes cross-module, generic and `Self` nesting, unknown
  collections, and qualified wrapper paths in one pass.
- **B8 + B13 + B14 + B9:** recursion end to end, across `settings()`, schema names, the JSON root
  ref and `Box<Self>`.
- **B18 + G5:** a schema-fidelity pass:
  - add `SchemaField.default` and `SchemaField.required`, and `StructType.deny_unknown_fields`;
  - add `transparent` and `skip_deserializing`;
  - add integer ranges;
  - optionally, validator-derived constraints.

  It touches every renderer, so do it as one change with snapshot review.
- **B19 + G6 + B17:** a renderer-validity pass. Run each template and the TS output through a real
  parser (toml, serde_norway, a TS checker) in tests, so validity is asserted, not just snapshotted.
- **X3–X7 + G4:** error location and provenance.
- **G1–G3:** loader and merge additions. They are independent and additive; G3 should start with
  the scalar-only newtype.

## Renderer feature matrix

✓ handled · ~ partial · ✗ missing or wrong · n/a not applicable

| Feature                     | JSON Schema         | TypeScript      | Templates      | Pkl schema     | API docs        |
| --------------------------- | ------------------- | --------------- | -------------- | -------------- | --------------- |
| doc `@tags`                 | ✗ raw in text       | ✓               | ~ raw lines    | ✓              | ✓               |
| deprecated message          | ~ bool only         | ✓               | ✓              | ✓              | ✓               |
| field default               | ~ leaks (B18)       | ~ leaks         | ~ leaks        | ~ alias leaks  | ~ leaks         |
| env var                     | ✗                   | ✓               | ✓              | ✗              | ✓               |
| `Option` field omittable    | ✗                   | ✗               | n/a            | ✓              | ✓               |
| `required` on `Option`      | ✗                   | ✗               | n/a            | ✗              | ✗               |
| aliases                     | ~                   | ✓               | ✗              | ✓              | ✓               |
| flatten map                 | ✓                   | ✓               | ✗              | ✓              | ✓               |
| flatten struct              | ✗ (B15)             | ✗ collision     | ✗ dropped      | ✓              | ✓               |
| internal tag + named struct | ✗ (B16)             | ✗               | ?              | ✓              | ✗               |
| recursion                   | ✗ root ref, ✗ `Box` | ✗ `Box`         | ✗ `Box`        | ✓              | ✗ `Box`         |
| integer kind and range      | ✗ `number`          | ~               | ✓              | ✓              | ~               |
| string/number constraints   | ✓                   | ✗               | ✗              | ✓              | ✓               |
| unknown fields allowed      | ✗ always closed     | n/a             | n/a            | ~              | ✗               |
| non-string map keys         | ✗                   | ~ strict Record | n/a            | ✓              | ✓               |
| `Unknown` / any             | ✗ excludes null     | ✓               | ✗ TOML `null`  | ✓              | ✓               |
| `transparent`               | ✗                   | ✗               | ✗              | ✗              | ✗               |
| identifier/string escaping  | ✓                   | ✗ (B17)         | ✗ (B19)        | ✓              | ✓               |

TypeScript also has smaller typing gaps:

- `Record<Enum, V>` requires every key; it should be `Partial<Record<..>>`.
- `EnumFormat::Enum` emits a numeric `enum`.
- `[T; N]` renders as `T[]`.
- `read_only` never becomes `readonly`.

## Checked and fine

- **Feature combinations:** all 22 tried build warning-free, and `cargo doc --all-features` is
  clean. Generated code compiles under `config` alone and with each of env, validate, extends and
  schema.
- **Parse errors:** JSON and TOML type-error spans are accurate. BOM stripping, JSONC comments and
  YAML merge keys work.
- **Validation:** aggregation and paths (`list[0].x`, `map.key.x`, variant paths) work, and
  `required` semantics behave as documented.
- **Extends:** relative paths resolve against the parent file, list order sets precedence, and
  formats can be mixed.
- **Env:** explicit env keys are absolute, derived prefixes work, the `split_*` helpers trim and
  skip empty items, and env errors name the variable.
- **Flatten:** a flattened nested config under `deny_unknown_fields` merges, applies defaults, and
  rejects unknown keys.
- **Enums:** internally tagged enums of nested configs load. Negative literal defaults, field
  `#[cfg]` and `Cow<'a, str>` work.
- **Unions:** derived code never emits `oneOf`; it uses `anyOf`, which suits overlapping unions.
  `anyOf [T, null]` nullability, draft-07 tuples, and external and adjacent tags in JSON and TS are
  correct.
- **Std types:** `Duration`, `SystemTime` and `Range` are modelled as maps, matching serde.
- **Partials:** partialization works (`Partial` renames, `Reference.partial`, TS `?: T | null`).
  Generic `schema_name` composition and the generator's name-collision panic work.
- **Pkl:** it handles recursion, alias cycles and the internal-tag class.
- **Orphan partial schema:** the bug noted in memory did not reproduce for `Point`,
  `Option<Point>` or `Vec<Point>` fields.
- **Not covered by any test:** raw identifiers, `VecDeque`, nesting a config from another module,
  qualified paths, `Box<Path>`, RON, and non-idempotent merges on nested configs.
