```rust
#[derive(Config)]
enum Host {
	#[setting(default)]
	Local,
	Remote(HostConfig),
}
```

> Enums only support `#[setting(default)]`, which denotes that variant as the default. It does not
> support setting values for the variant itself, or its inner tuple fields.

```
#[derive(Config)]
enum Projects {
	#[setting(merge = schematic::merge::append_vec)]
	List(Vec<String>),
	// ...
}
```

Or on a per-variant basis when using an enum.

```rust
#[derive(Config)]
enum Projects {
	#[setting(validate = schematic::validate::min_length(1))]
	List(Vec<String>),
	// ...
}
```
