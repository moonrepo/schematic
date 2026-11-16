# Docs drift

From the 2026-11-14 audit. See [README.md](README.md) for the legend, the ID scheme, the top
priorities, and the quick wins.

- **Book feature names:** `book/src/schema/generator/{json-schema,typescript,template}.md:3` name
  the features `json_schema`, `typescript` and `template`. The real names are `renderer_*`.
- **Book required fields:** `generator/json-schema.md` says only non-`Option` fields are required,
  but the renderer marks `Option` fields required too (G5).
- **Book layer type:** `book/src/config/index.md:101` says `Vec<Layer<PartialAppConfig>>`; the type
  is `Vec<Layer<AppConfig>>`.
- **Book `mut`:** `index.md:71` and `:85` call `.code()` on a non-`mut` binding.
- **Book JSON env strings:** `book/src/config/struct/env.md:84` mentions them, but no helper exists.
- **Book struct syntax:** `book/src/schema/index.md` struct examples use `pub name: String;`.
- **Book chrono types:** `book/src/schema/external.md` lists chrono types that have no serde support.
- **README formats:** the README implies formats work out of the box (X10). Fixed on `0.21-devx`.
