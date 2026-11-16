# Validation rules

> Requires the `validate` Cargo feature, which is enabled by default.

What kind of configuration crate would this be without built-in validation? As such, we support it
as a first-class feature, with built-in validation rules provided by the
[garde](https://crates.io/crates/garde) crate.

In Schematic, validation _does not_ happen as part of the serde parsing process, and instead happens
for each [partial configuration](../partial.md) to be merged. However, with that said, _prefer serde
parsing over validation rules for structural adherence_
([learn more](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)).

Validation can be applied on a per-setting basis with the `#[setting(validate)]` attribute field,
which requires a reference to a function to call.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(validate = schematic::validate::alphanumeric)]
	pub secret_key: String,

	#[setting(validate = schematic::validate::regex("^\.env"))]
	pub env_file: String,
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

A variant holding several values passes them as a tuple of references, not as a reference to a
tuple.

```rust
fn validate_pair<P, C>(value: (&String, &usize), partial: &P, context: &C, finalize: bool) -> ValidateResult {
	// ...
}

#[derive(Config)]
enum Entry {
	#[setting(validate = validate_pair)]
	Pair(String, usize),
	// ...
}
```

> We provide a handful of built-in validation functions in the
> [`validate` module](https://docs.rs/schematic/latest/schematic/validate/index.html). Furthermore,
> some functions are factories which can be called to produce a validator, like `min(1)`, `max(10)`,
> and `in_range(1, 10)` for numbers, or `min_length(1)` for strings and lists.

## Validation passes

A configuration is validated more than once while loading. Each source is validated on its own as
it's parsed, and the final configuration is validated again once every source,
[default value](./default.md), and [environment variable](./env.md) has been merged, and every
[transform](./transform.md) has run. A validator is told which pass it's in by its last argument,
`finalizing`, which is only `true` for the final pass.

A source only holds part of the configuration, and a later one may replace a value an earlier one
set, so a check that needs the final value should wait for the final pass. For example, a path that
a transform makes absolute isn't absolute yet when its source is validated.

```rust
fn absolute_path<P, C>(value: &str, partial: &P, context: &C, finalizing: bool) -> ValidateResult {
	if finalizing && !Path::new(value).is_absolute() {
		return Err(ValidateError::new("must be an absolute path"));
	}

	Ok(())
}
```

> Settings that no source provided are `None` in each source's pass, and are skipped, while the
> final pass has a value for every setting.

An error in a source's pass names that source, like `Failed to validate config.yml`. The final pass
validates what every layer merged into, so its error names each source, and the environment
variables when any were set, like `Failed to validate base.yml, config.yml, environment variables`,
or the config's name when there are neither.

## Required settings

Every setting in a [partial configuration](../partial.md) is optional, so a setting that no source
provides falls back to its [default value](./default.md), or to `Default::default()` when it has
none. To fail instead, as serde does for a missing field, mark the setting with
`#[setting(required)]`.

```rust
#[derive(Config)]
struct AppConfig {
	#[setting(required)]
	pub name: String,

	#[setting(required, env = "APP_TOKEN")]
	pub token: String,

	#[setting(required)]
	pub port: Option<usize>,
}
```

A required setting has no default value, and is checked by the final validation pass, once every
source and [environment variable](./env.md) has been merged. Any one of them may provide it, so each
source is free to omit it. A `null` counts as missing, while an empty value, like `""` or `[]`, does
not.

When the setting is wrapped in `Option`, the value must be `Some`, but the field stays an `Option`
in the final configuration. Its [schema](../../schema/struct.md#optional-and-nullable-fields)
describes the type the `Option` holds, as `null` isn't a value. A setting that isn't wrapped still has to implement `Default`, which
`AppConfig::default()` falls back to.

A required setting can't declare a `default`, as it would always have a value. Nor can a
[nested config](../nested.md) that isn't wrapped in `Option`, as one always exists in the final
configuration. Mark the settings within it as required instead, which are then checked even when no
source provided the nested config at all.

## Validate handler function

You can also define your own function for validating values, also known as a validator.

When defining a custom validate function, the value to check is passed as the first argument, the
current/parent partial as the second, the [context](../context.md) as the third, and whether this is
the final validation pass.

```rust
fn validate_string(
	value: &str,
	partial: &PartialAppConfig,
	context: &Context
	finalize: bool
) -> ValidateResult {
	if !do_check(value) {
		return Err(ValidateError::new("Some failure message"));
	}

	Ok(())
}
```

If validation fails, you must return a
[`ValidateError`](https://docs.rs/schematic/latest/schematic/struct.ValidateError.html) with a
failure message.

The setting is passed by reference, and deref coerces on its way in, so the first argument can take
a borrowed form of the type instead of the type itself. A `String` setting reaches a `&str`
validator, as above, and a `Vec<T>` setting reaches a `&[T]` one.

### Factories

For composition and reusability concerns, we also support factory functions that can be called to
create a unique validator. This can be seen above with `schematic::validate::regex`. To create your
own factory, declare a normal function, with any number of arguments, that returns a
[`Validator`](https://docs.rs/schematic/latest/schematic/type.Validator.html).

Using the `regex` factory as an example, it would look something like this. The closure takes the
same four arguments a plain validator does.

```rust
use schematic::Validator;

fn regex<T, P, C>(pattern: &str) -> Validator<T, P, C> {
	let pattern = regex::Regex::new(pattern).unwrap();

	Box::new(move |value, _, _, _| {
		if !pattern.is_match(value) {
			return Err(ValidateError::new("Some failure message"));
		}

		Ok(())
	})
}
```

### Several errors

A validator returns one error, but may return several instead, such as one for each item of a list
that fails, by returning a `Vec<ValidateError>`, or a garde `Report`. Anything that implements
[`IntoValidateErrors`](https://docs.rs/schematic/latest/schematic/trait.IntoValidateErrors.html)
can be returned.

```rust
fn no_empty_items<P, C>(items: &[String], partial: &P, context: &C, finalizing: bool) -> Result<(), Vec<ValidateError>> {
	let errors = items
		.iter()
		.enumerate()
		.filter(|(_, item)| item.is_empty())
		.map(|(index, _)| ValidateError::with_segment("must not be empty", PathSegment::Index(index)))
		.collect::<Vec<_>>();

	if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### Path targeting

If validating an item in a list or collection, you can specifiy the nested path when failing. This
is extremely useful when building error messages.

```rust
use schematic::PathSegment;

ValidateError::with_segments(
	"Some failure message",
	// [i].key
	[PathSegment::Index(i), PathSegment::Key(key.to_string())]
)
```

### Context and partial handling

If you're not using [context](../context.md), or want to create a validator for any kind of partial,
we suggest generic inferrence.

```rust
fn using_generics<P, C>(value: &str, partial: &P, context: &C, finalize: bool) -> ValidateResult {
	// ...
}
```

## Config validation

A rule that compares settings, rather than checking one, can validate the configuration as a whole,
with the `#[config(validate)]` container attribute field. The function receives the partial
configuration, the [context](../context.md), and whether this is the
[final pass](#validation-passes), and may return [several errors](#several-errors). It runs after the
validators of each setting, for structs and enums alike, and is a factory when called, like
`#[config(validate = at_most(10))]`.

```rust
fn min_below_max<C>(partial: &PartialRangeConfig, context: &C, finalizing: bool) -> ValidateResult {
	// A source may only set one of them
	if let (Some(min), Some(max)) = (partial.min, partial.max)
		&& min > max
	{
		return Err(ValidateError::with_segment(
			"must be at most max",
			PathSegment::Key("min".into()),
		));
	}

	Ok(())
}

#[derive(Config)]
#[config(validate = min_below_max)]
struct RangeConfig {
	pub min: usize,
	pub max: usize,
}
```

An error is reported at the configuration's own path, like `range` when it's nested, unless it
[targets a setting](#path-targeting) within it, like `range.min`.

## Cargo features

The following Cargo features can be enabled for more functionality:

- `validate_email` - Enables email validation with the `schematic::validate::email` function.
- `validate_url` - Enables URL validation with the `schematic::validate::url` and `url_secure`
  functions.
