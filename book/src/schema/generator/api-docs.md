# API documentation

> Requires the `renderer_api_docs` Cargo feature.

With our
[`ApiDocsRenderer`](https://docs.rs/schematic/latest/schematic/schema/api_docs/struct.ApiDocsRenderer.html),
you can generate markdown API documentation for all types that implement
[`Schematic`](https://docs.rs/schematic/latest/schematic/schema/trait.Schematic.html). Each type
becomes its own page, describing every property or variant, and linking to the pages of the types it
references.

To utilize, instantiate a generator, add types to render, and generate the output file.

```rust
use schematic::schema::{SchemaGenerator, ApiDocsRenderer};

let mut generator = SchemaGenerator::default();
generator.add::<CustomType>();
generator.generate(output_dir.join("CustomType.md"), ApiDocsRenderer::default())?;
```

## One page per type

Like a [JSON schema](./json-schema.md), each render produces a single document, and the _last type
to be added to `SchemaGenerator`_ is the one rendered. Every other type in the generator is known to
the renderer, so a property whose type is one of them links to that type's page.

Adding a type that is already in the generator, because an earlier type nested it, moves it to the
end. To generate a page for every type, add each one before generating its file.

```rust
let mut generator = SchemaGenerator::default();

generator.add::<RootConfig>();
generator.generate(output_dir.join("RootConfig.md"), ApiDocsRenderer::default())?;

// Nested by `RootConfig`, so already in the generator
generator.add::<NestedConfig>();
generator.generate(output_dir.join("NestedConfig.md"), ApiDocsRenderer::default())?;
```

Links are relative, so all pages are expected to live in the same directory, and to be named after
the type they document.

## Page structure

A page starts with front matter that titles it after the type, followed by the type's description.
A struct then lists its properties, a unit enum lists its variants, and a union lists each type it
accepts. Types that are referenced from the page are listed at the end.

```markdown
---
title: ServerConfig
---

Configures the HTTP server.

## Properties

### `port`

> **Optional**

The port to listen on.

| Attribute | Value |
| --- | --- |
| Type | `number` |
| Default | `8080` |
| Minimum | `1024` |
| Environment variable | `SERVER_PORT` |

### `tls`

> **Nullable**

TLS settings, when serving over HTTPS.

| Attribute | Value |
| --- | --- |
| Type | [`TlsConfig`](./TlsConfig.md) |

## References

- [TlsConfig](./TlsConfig.md)
```

The tags after a name describe how a property may be provided: whether it is required or optional,
nullable, deprecated (with the deprecation message when there is one), read only, write only, or
flattened into its parent. Enum variants are tagged with whether they are the default.

The table describes the type itself: its shape, default value, accepted values, constraints such as
minimums and patterns, aliases, and the environment variable it can be set with.

## Options

Custom options can be passed to the renderer using
[`ApiDocsOptions`](https://docs.rs/schematic/latest/schematic/schema/api_docs/struct.ApiDocsOptions.html).

```rust
use schematic::schema::ApiDocsOptions;

ApiDocsRenderer::new(ApiDocsOptions {
	// ...
	..ApiDocsOptions::default()
});
```

### Link extension

Links to other pages append `.md` to the type name. Documentation tools that route on the file name
rather than the file may want bare links instead, which the `link_extension` option controls.

```rust
ApiDocsOptions {
	// ...
	link_extension: "".into(),
}
```

### Required fields

When a struct is rendered, fields that are not optional, nullable, or flattened are tagged as
required. This is enabled by default.

```rust
ApiDocsOptions {
	// ...
	mark_struct_fields_required: false,
}
```

### Aliases

A field's aliases are listed in its table. Disable this with the `exclude_aliases` option.

```rust
ApiDocsOptions {
	// ...
	exclude_aliases: true,
}
```
