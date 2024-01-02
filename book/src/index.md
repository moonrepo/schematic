# Schematic

Schematic is a library that provides:

- A layered serde-driven [configuration system](./config/index.md) with support for merge
  strategies, validation rules, environment variables, and more!
- A [schema modeling system](./schema/index.md) that can be used to generate TypeScript types, JSON
  schemas, and more!

Both of these features can be used independently or together.

```
cargo add schematic
```

## Example references

The following projects are using Schematic and can be used as a reference:

- [moon](https://github.com/moonrepo/moon/tree/master/nextgen/config) - A build system for web based
  monorepos.
- [proto](https://github.com/moonrepo/proto/blob/master/crates/core/src/proto_config.rs) - A
  multi-language version manager with WASM plugin support.
- [ryot](https://github.com/IgnisDa/ryot/blob/main/libs/config/src/lib.rs) - Track various aspects
  of your life.
