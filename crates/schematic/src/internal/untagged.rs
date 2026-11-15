/// Buffer the content of an untagged enum, so that each of its variants can
/// attempt to deserialize it. A format reports `null` as a unit, which
/// `serde_content` deserializes as a present value, failing every `Option`
/// within a variant, so a unit is buffered as none instead. Serde's own
/// buffering does the same.
pub fn buffer_untagged_content<'de, D>(
    deserializer: D,
) -> Result<serde_content::Value<'de>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_content::Value;

    fn nullify(value: Value<'_>) -> Value<'_> {
        match value {
            Value::Unit => Value::Option(None),
            Value::Map(entries) => Value::Map(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, nullify(value)))
                    .collect(),
            ),
            Value::Option(Some(inner)) => Value::Option(Some(Box::new(nullify(*inner)))),
            Value::Seq(items) => Value::Seq(items.into_iter().map(nullify).collect()),
            Value::Tuple(items) => Value::Tuple(items.into_iter().map(nullify).collect()),
            other => other,
        }
    }

    deserializer
        .deserialize_any(serde_content::ValueVisitor)
        .map(nullify)
}

/// Return true if the buffered content of an untagged enum is `null`, which a
/// unit variant is represented as.
pub fn is_null_content(content: &serde_content::Value<'_>) -> bool {
    matches!(
        content,
        serde_content::Value::Unit | serde_content::Value::Option(None)
    )
}

/// Deserialize the buffered content of an untagged enum as the value of one
/// of its variants. On failure, the error is prefixed with the path to the
/// setting that failed within the variant, as the path the loader reports
/// ends at the enum itself.
pub fn deserialize_untagged_variant<'de, T>(
    content: &serde_content::Value<'de>,
) -> Result<T, String>
where
    T: serde::Deserialize<'de>,
{
    use serde_path_to_error::Segment;

    let deserializer = serde_content::Deserializer::new(content.clone())
        .coerce_numbers()
        .human_readable();

    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let message = describe_content_error(error.inner());

        // Mirrors how `serde_path_to_error` itself displays an error
        if error
            .path()
            .iter()
            .all(|segment| matches!(segment, Segment::Unknown))
        {
            message
        } else {
            format!("{}: {message}", error.path())
        }
    })
}

/// Return the buffered content of a literal union as a string, which a
/// named unit variant is written as.
pub fn literal_content<'a>(content: &'a serde_content::Value<'_>) -> Option<&'a str> {
    match content {
        serde_content::Value::String(value) => Some(value),
        _ => None,
    }
}

/// Describe why the buffered content of a literal union matched none of
/// its named values.
pub fn literal_error_message(values: &[&str], content: &serde_content::Value<'_>) -> String {
    use serde_content::{ErrorKind, Expected, Unexpected};

    let expected = match values {
        [value] => format!("{value:?}"),
        _ => format!(
            "one of {}",
            values
                .iter()
                .map(|value| format!("{value:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };

    // `serde_content` only exposes what it found through an error
    match content.clone().unexpected(Expected::String).kind() {
        ErrorKind::Unexpected { found, .. } => {
            format!("expected {expected}, found {}", describe_found(found))
        }
        _ => format!("expected {expected}"),
    }
}

/// Combine the errors of every variant of an untagged enum into a single
/// message, as serde's own discards them. Each error is indented beneath its
/// variant, so that a nested untagged enum lists its variants beneath the
/// variant that holds it, rather than alongside. An error without a variant
/// stands for several, like the named values of a literal union, and lists
/// them itself.
pub fn untagged_error_message(name: &str, errors: &[(&str, String)]) -> String {
    let mut message = format!("failed to parse as any variant of {name}:");

    for (variant, error) in errors {
        message.push_str("\n- ");

        if !variant.is_empty() {
            message.push_str(variant);
            message.push_str(": ");
        }

        message.push_str(&error.replace('\n', "\n  "));
    }

    message
}

fn describe_content_error(error: &serde_content::Error) -> String {
    match error.kind() {
        // Displayed by `serde_content` with a "failed to deserialize" prefix,
        // and the found value in full, which for a map is the entire input
        serde_content::ErrorKind::Unexpected { found, expected } => {
            format!("expected {expected}, found {}", describe_found(found))
        }
        _ => error.to_string(),
    }
}

/// Describe a found value the way it was written in the source, so a scalar
/// shows its value (without the Rust type suffix `serde_content` adds), and a
/// collection only its kind.
fn describe_found(found: &serde_content::Found) -> String {
    use serde_content::{Found, Number};

    match found {
        Found::Unit | Found::Option(None) => "null".into(),
        Found::Option(Some(inner)) => describe_found(inner),
        Found::Bool(value) => value.to_string(),
        Found::Number(number) => match number {
            Number::I8(value) => value.to_string(),
            Number::U8(value) => value.to_string(),
            Number::I16(value) => value.to_string(),
            Number::U16(value) => value.to_string(),
            Number::I32(value) => value.to_string(),
            Number::U32(value) => value.to_string(),
            Number::I64(value) => value.to_string(),
            Number::U64(value) => value.to_string(),
            Number::I128(value) => value.to_string(),
            Number::U128(value) => value.to_string(),
            // Debug keeps the decimal point, so `1.0` isn't shown as `1`
            Number::F32(value) => format!("{value:?}"),
            Number::F64(value) => format!("{value:?}"),
            _ => "a number".into(),
        },
        Found::Char(value) => format!("{:?}", value.to_string()),
        Found::String(value) | Found::Identifier(value) => format!("{value:?}"),
        Found::Bytes(_) => "bytes".into(),
        Found::Seq(_) | Found::Tuple(_) => "a list".into(),
        Found::Map(_) | Found::Struct { .. } => "a map".into(),
        Found::Enum { .. } => "an enum variant".into(),
    }
}
