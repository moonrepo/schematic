# Default variant

To define a default variant, use the `Default` trait and the optional `#[default]` variant
attribute. We provide no special functionality or syntax for handling defaults.

```rust
#[derive(ConfigEnum, Default)]
enum LogLevel {
	Info,
	Error,
	Debug,
	#[default]
	Off
}
```
