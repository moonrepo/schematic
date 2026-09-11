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
the renderer, so a property whose type is one of them links to that type's page. A composite type,
such as a list of a config, is rendered as code, which cannot hold a link, so the types it refers to
are linked in a references row of their own.

To generate a page for every type at once, including the ones that were added by nesting, use
[`ApiDocsRenderer::generate_all()`](https://docs.rs/schematic/latest/schematic/schema/api_docs/struct.ApiDocsRenderer.html#method.generate_all)
with an output directory instead. It writes a page per type, named after the type with a `.md`
extension, along with an [index page](#index-page).

```rust
let mut generator = SchemaGenerator::default();
generator.add::<RootConfig>();

ApiDocsRenderer::default().generate_all(&generator, output_dir.join("config"))?;
```

To control each page individually, generate them one at a time. Adding a type that is already in
the generator, because an earlier type nested it, moves it to the end, so add each one before
generating its file.

```rust
generator.add::<RootConfig>();
generator.generate(output_dir.join("RootConfig.md"), ApiDocsRenderer::default())?;

// Nested by `RootConfig`, so already in the generator
generator.add::<NestedConfig>();
generator.generate(output_dir.join("NestedConfig.md"), ApiDocsRenderer::default())?;
```

By default links are relative, so all pages are expected to live in the same directory, and to be
named after the type they document, which is what `generate_all` produces. The
[`render_link`](#custom-links) option changes that, and a custom link should agree with wherever
the pages are written.

## Index page

`generate_all` also writes an index page, a table of every type that links to its page, with its
kind and the first paragraph of its description.

```markdown
---
title: Index
---

## Types

| Type | Kind | Description |
| --- | --- | --- |
| [`ServerConfig`](./ServerConfig.md) | Struct | Configures the HTTP server. |
| [`TlsConfig`](./TlsConfig.md) | Struct | TLS settings, when serving over HTTPS. |
```

The `index_page` option names the file, and `None` skips it. When generating pages one at a time,
[`ApiDocsRenderer::render_index_page()`](https://docs.rs/schematic/latest/schematic/schema/api_docs/struct.ApiDocsRenderer.html#method.render_index_page)
renders the same content from the generator's schemas.

```rust
ApiDocsOptions {
	// ...
	index_page: Some("README.md".into()),
}
```

## Page structure

A page starts with frontmatter that titles it after the type, followed by the type's description,
and an index that summarizes every section to come, linking to each. A struct then lists its
properties, and an enum lists its variants, headed by the variant name. A union that was built by
hand has no variant names, so each of its variants is headed by its type instead. Types that are
referenced from the page are listed at the end.

```markdown
---
title: ServerConfig
---

Configures the HTTP server.

## Index

| Property        | Type                          | Description                            |
| --------------- | ----------------------------- | -------------------------------------- |
| [`port`](#port) | `number`                      | The port to listen on.                 |
| [`tls`](#tls)   | [`TlsConfig`](./TlsConfig.md) | TLS settings, when serving over HTTPS. |

## Properties

### `port`

> **Optional**

The port to listen on.

| Attribute            | Value         |
| -------------------- | ------------- |
| Type                 | `number`      |
| Default              | `8080`        |
| Minimum              | `1024`        |
| Environment variable | `SERVER_PORT` |

### `tls`

> **Nullable**

TLS settings, when serving over HTTPS.

| Attribute | Value                         |
| --------- | ----------------------------- |
| Type      | [`TlsConfig`](./TlsConfig.md) |

## References

- [`TlsConfig`](./TlsConfig.md)
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

### Frontmatter

Each page opens with frontmatter that titles it after the type. Documentation tools often read more
from it, such as a sidebar position or a label, which the `frontmatter` map adds as `key: value`
lines after the title, in key order. Values are written verbatim, so quote them as the tool expects.
A `title` entry replaces the type name.

```rust
use std::collections::BTreeMap;

ApiDocsOptions {
	// ...
	frontmatter: BTreeMap::from_iter([
		("sidebar_position".into(), "2".into()),
		("description".into(), "\"Configures the server.\"".into()),
	]),
}
```

The renderer is created for each page, so per-type values can be set when generating that type.

### Custom links

Links to other pages are rendered by the `render_link` function, which receives the name of the
referenced type and returns the whole link, label included. The default labels the link with the
name as inline code and links to a markdown file named after the type in the same directory, such
as `` [`Name`](./Name.md) ``. Documentation tools that route on a path rather than a file, pages
split across directories, or a site with its own link component can return whatever they need. The
same function renders the inline links and the references list.

```rust
ApiDocsOptions {
	// ...
	render_link: Box::new(|name| format!("<Link to=\"/docs/config/{name}\">{name}</Link>")),
}
```

### Enum format

A unit enum renders a section per variant, which is thorough but long for an enum with many
variants. The `enum_format` option can render them as a single table instead, one row per variant
with its value, tags, and description. The index is omitted for it, as the table already summarizes
every variant. Unions always render as sections, since their variants carry values that need
describing.

```rust
use schematic::schema::ApiDocsEnumFormat;

ApiDocsOptions {
	// ...
	enum_format: ApiDocsEnumFormat::Table,
}
```

```markdown
## Variants

| Variant | Value     | Tags                                 | Description            |
| ------- | --------- | ------------------------------------ | ---------------------- |
| `debug` | `"debug"` |                                      | Log everything.        |
| `info`  | `"info"`  | **Default**                          | Log at info and above. |
| `off`   | `"off"`   | **Deprecated** (Use `none` instead.) | Disable logging.       |
```

> Tags in the table are rendered as labels, not through the `render_tags` function, as its output is
> a block that cannot sit inside a table cell.

### Index

The index ahead of the properties or variants shows each one's type and the first paragraph of its
description. Disable it with the `include_index` option.

```rust
ApiDocsOptions {
	// ...
	include_index: false,
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

### Custom tags

Tags are rendered by the `render_tags` function, which receives the tags of a type, property, or
variant as a list of
[`ApiDocsTag`](https://docs.rs/schematic/latest/schematic/schema/api_docs/enum.ApiDocsTag.html)
values, and returns markdown. The default renders a block quote of bold labels, but a documentation
site may prefer its own components.

```rust
use schematic::schema::ApiDocsTag;

ApiDocsOptions {
	// ...
	render_tags: Box::new(|tags| {
		tags.iter()
			.map(|tag| match tag {
				ApiDocsTag::Deprecated(Some(message)) => format!("<Badge>{tag}</Badge> {message}"),
				_ => format!("<Badge>{tag}</Badge>"),
			})
			.collect::<Vec<_>>()
			.join(" ")
	}),
}
```

The function is only called when there is at least one tag, and returning an empty string omits the
tags entirely.

### Custom descriptions

Descriptions are rendered by the `render_description` function, which receives the doc comment as
written and returns markdown. The default keeps the comment as is, only trimming each line. Use this
to wrap descriptions in a component, or to rewrite links.

```rust
ApiDocsOptions {
	// ...
	render_description: Box::new(|description| format!(":::note\n{}\n:::", description.trim())),
}
```

Returning an empty string omits the description.

### Aliases

A field's aliases are listed in its table. Disable this with the `exclude_aliases` option.

```rust
ApiDocsOptions {
	// ...
	exclude_aliases: true,
}
```
