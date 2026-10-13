#[cfg(feature = "env")]
mod env;
mod merge;
#[cfg(feature = "validate")]
mod validate;

#[cfg(feature = "env")]
pub use env::*;
pub use merge::*;
#[cfg(feature = "validate")]
pub use validate::*;

use crate::config::{ConfigError, HandlerError, MergeError, MergeResult, PartialConfig};
use schematic_types::Schema;
use std::str::FromStr;

// UNTAGGED ENUMS

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

/// Combine the errors of every variant of an untagged enum into a single
/// message, as serde's own discards them. Each error is indented beneath its
/// variant, so that a nested untagged enum lists its variants beneath the
/// variant that holds it, rather than alongside.
pub fn untagged_error_message(name: &str, errors: &[(&str, String)]) -> String {
    let mut message = format!("failed to parse as any variant of {name}:");

    for (variant, error) in errors {
        message.push_str("\n- ");
        message.push_str(variant);
        message.push_str(": ");
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

// CASING

/// Normalize a string into the provided case, using the same format names as
/// serde's `rename_all`. Backs `#[config(before_parse)]`, where the incoming
/// value is reshaped before it is matched against a variant.
///
/// The format is validated at derive time, so an unknown one is unreachable
/// from generated code.
pub fn format_case(value: &str, format: &str) -> String {
    use convert_case::{Boundary, Case, Casing};

    let case = match format {
        // `Case::Lower`/`Case::Upper` are space delimited, so these two are
        // handled directly rather than through a case conversion
        "lowercase" => return value.to_lowercase(),
        "UPPERCASE" => return value.to_uppercase(),
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "kebab-case" => Case::Kebab,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        other => {
            panic!("Unknown `before_parse` value `{other}`.");
        }
    };

    value
        // Unlike the derive-time equivalent this does not hint at an incoming
        // case, since the value came from a user rather than a Rust ident
        .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(case)
}

// DEFAULT VALUES

pub fn handle_default_result<T, E: std::error::Error>(
    result: Result<T, E>,
) -> Result<T, ConfigError> {
    result.map_err(|error| ConfigError::InvalidDefaultValue(error.to_string()))
}

// LEGACY

#[cfg(feature = "env")]
pub fn track_env<T>(value: Option<T>, tracker: &mut std::collections::HashSet<bool>) -> Option<T> {
    value.inspect(|_| {
        tracker.insert(true);
    })
}

#[cfg(feature = "env")]
pub fn default_env_value<T: FromStr>(key: &str) -> crate::config::ParseEnvResult<T> {
    parse_env_value(key, |value| parse_value(value).map(|v| Some(v)))
}

#[cfg(feature = "env")]
pub fn parse_env_value<T>(
    key: &str,
    parser: impl Fn(String) -> crate::config::ParseEnvResult<T>,
) -> crate::config::ParseEnvResult<T> {
    if let Ok(value) = std::env::var(key) {
        return parser(value)
            .map_err(|error| HandlerError(format!("Invalid environment variable {key}. {error}")));
    }

    Ok(None)
}

pub fn parse_value<T: FromStr, V: AsRef<str>>(value: V) -> Result<T, HandlerError> {
    let value = value.as_ref();

    value.parse::<T>().map_err(|_| {
        HandlerError(format!(
            "Failed to parse \"{value}\" into the correct type."
        ))
    })
}

pub fn merge_setting<T, C>(
    prev: Option<T>,
    next: Option<T>,
    context: &C,
    merger: impl Fn(T, T, &C) -> MergeResult<T>,
) -> MergeResult<T> {
    match (prev, next) {
        (Some(prev), Some(next)) => merger(prev, next, context),
        (None, Some(next)) => Ok(Some(next)),
        (other, _) => Ok(other),
    }
}

pub fn merge_nested_setting<T: PartialConfig>(
    prev: Option<T>,
    next: Option<T>,
    context: &T::Context,
) -> MergeResult<T> {
    match (prev, next) {
        (Some(mut prev), Some(next)) => {
            prev.merge(context, next)
                .map_err(|error| MergeError(error.to_string()))?;

            Ok(Some(prev))
        }
        (None, Some(next)) => Ok(Some(next)),
        (other, _) => Ok(other),
    }
}

pub fn partialize_schema(schema: &mut Schema, force_partial: bool) {
    use schematic_types::*;

    let mut update_name = |update: bool| {
        if update {
            if let Some(name) = &schema.name {
                if !name.starts_with("Partial") {
                    schema.name = Some(format!("Partial{name}"));
                }
            }
        }
    };

    match &mut schema.ty {
        SchemaType::Array(inner) => {
            partialize_schema(&mut inner.items_type, false);
        }
        SchemaType::Object(inner) => {
            partialize_schema(&mut inner.key_type, false);
            partialize_schema(&mut inner.value_type, false);
        }
        SchemaType::Struct(inner) => {
            if inner.partial || force_partial {
                update_name(true);

                for field in inner.fields.values_mut() {
                    field.optional = true;
                    field.nullable = true;
                    field.schema.nullify();

                    partialize_schema(&mut field.schema, true);
                }
            } else {
                for field in inner.fields.values_mut() {
                    partialize_schema(&mut field.schema, false);
                }
            }
        }
        SchemaType::Tuple(inner) => {
            for item in inner.items_types.iter_mut() {
                partialize_schema(item, false);
            }
        }
        SchemaType::Union(inner) => {
            update_name(inner.partial || force_partial);

            for variant in inner.variants_types.iter_mut() {
                partialize_schema(variant, false);
            }
        }
        // A cycle within a nested config resolves to a reference, which must
        // follow the type it points at, otherwise it names a type that was
        // never rendered.
        SchemaType::Reference { name, partial }
            if (*partial || force_partial) && !name.starts_with("Partial") =>
        {
            *name = format!("Partial{name}");
        }
        _ => {}
    };
}
