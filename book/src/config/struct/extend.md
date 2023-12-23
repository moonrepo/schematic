# Extendable sources

> Not supported for enums.

Configs can extend other configs, generating an accurate layer chain, via the `#[setting(extend)]`
attribute field. Extended configs can either be a file path (relative from the current config) or a
secure URL.

When defining `extend`, we currently support 3 types of patterns. We also suggest making the setting
optional, so that extending is not required by consumers!

## Single source

The first pattern is with a single string, which only allows a single file or URL to be extended.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_string)]
	extends: Option<String>,
}
```

Example:

```yaml
extends: "./another/file.yml"
```

## Multiple sources

The second pattern is with a list of strings, allowing multiple files or URLs to be extended. Each
item in the list is merged from top to bottom (lowest precedence to highest).

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_list)]
	extends: Option<Vec<String>>,
}
```

Example:

```yaml
extends:
  - "./another/file.yml"
  - "https://domain.com/some/other/file.yml"
```

## Either pattern

And lastly, supporting both a string or a list, using our built-in enum.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(extend, validate = schematic::validate::extends_from)]
	extends: Option<schematic::ExtendsFrom>,
}
```
