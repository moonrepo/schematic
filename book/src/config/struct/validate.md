# Validation rules

What kind of configuration crate would this be without built-in validation? As such, we support it
as a first-class feature, with built-in validation rules provided by
[garde](https://crates.io/crates/garde).

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

> We provide a handful of built-in validation functions in the
> [`validate` module](https://docs.rs/schematic/latest/schematic/validate/index.html). Furthermore,
> some functions are factories which can be called to produce a validator.

## Validate function

You can also define your own function for validating values, also known as a validator.

When defining a custom validate function, the value to check is passed as the first argument, the
current/parent partial as the second, and the [context](../context.md) as the third.

```rust
use schematic::ValidateError;

fn validate_string(
	value: &str,
	partial: &PartialAppConfig,
	context: &Context
) -> Result<(), ValidateError> {
	if !do_check(value) {
		return Err(ValidateError::new("Some failure message"));
	}

	Ok(())
}
```

If validation fails, you must return a `ValidateError` with a failure message.

### Factories

For composition and reusability concerns, we also support factory functions that can be called to
create a unique validator. This can be seen above with `schematic::validate::regex`. To create your
own factory, declare a normal function, with any number of arguments, that returns a `Validator`.

Using the `regex` factory as an example, it would look something like this.

```rust
use schematic::Validator;

fn regex<T, P, C>(pattern: &str) -> Validator<T, P, C> {
	let pattern = regex::Regex::new(pattern).unwrap();

	Box::new(move |value, _, _| {
		if !pattern.is_match(value) {
			return Err(ValidateError::new("Some failure message"));
		}

		Ok(())
	})
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
fn using_generics<P, C>(value: &str, partial: &P, context: &C) -> Result<(), ValidateError> {
	// ...
}
```
