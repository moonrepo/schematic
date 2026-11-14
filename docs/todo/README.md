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

Ordered by user impact per unit of work. Every bug was fixed on `0.21-bugs`.

1. **Turn derive errors into spanned compile errors (X1).** Today a typo in an attribute prints a raw
   `Debug` dump of the darling error.
2. **Make loader errors name the right place (X3–X6).** TOML syntax errors have no file, YAML type
   errors have no span, and validation is blamed on the last file.
3. **Ship the small loader and merge additions users already hand-roll (G1–G3).** A CLI or
   in-memory layer, the merge helpers moon wrote, and a runtime merge strategy (#175).

## Quick wins

Each item is trivial or a few lines, and every one is confirmed:

1. Map TOML syntax errors to `ParserError` (X3).
2. Report darling errors with `write_errors()` (X1).
3. Fall back to `type_name` for the loader name (X6).
4. Add `Debug` for `ConfigLoadResult`.
5. Skip schema inference for `skip` fields (C9).
6. Fix the merge built-in bounds and add generic hashers (G2).
7. Fix the book feature names.

## Larger changes worth sequencing together

- **G5:** a schema-fidelity pass, building on `SchemaField.default`:
  - add `SchemaField.required` and `StructType.deny_unknown_fields`;
  - add `transparent`;
  - add integer ranges;
  - optionally, validator-derived constraints.

  It touches every renderer, so do it as one change with snapshot review.
- **G6:** inline flattened fields in templates. `renderer_fixes_test.rs` already parses each template
  with a real parser, so extend it rather than relying on snapshots.
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
