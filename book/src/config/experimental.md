# Experimental

## Pkl configuration format (>= v0.17)

Thanks to the [`rpkl` crate](https://crates.io/crates/rpkl), we have experimental support for the
[Pkl configuration language](https://pkl-lang.org/index.html). Pkl is a dynamic and programmable
configuration format built and maintained by Apple.

```pkl
port = 3000
secure = true
allowedHosts = List(".localhost")
```

> Pkl support can be enabled with the `pkl` Cargo feature.

### Caveats

Unlike our other static formats, Pkl requires the following to work correctly:

- The `pkl` binary must exist on `PATH`. This requires every user to
  [install Pkl](https://pkl-lang.org/main/current/pkl-cli/index.html#installation) onto their
  machine.
- Pkl parses local file system paths only.
  - Passing source code directly to [`ConfigLoader`][loader] is NOT supported.
  - Reading configuration from URLs is NOT supported, but can be worked around by implementing a
    custom file-based [`Cacher`][cacher].

[cacher]: https://docs.rs/schematic/latest/schematic/trait.Cacher.html
[loader]: https://docs.rs/schematic/latest/schematic/struct.ConfigLoader.html

### Known issues

- The `rpkl` crate is relatively new and may be buggy or have missing/incomplete functionality.
- When parsing fails and a code snippet is rendered in the terminal using `miette`, the line/column
  offset may not be accurate.
