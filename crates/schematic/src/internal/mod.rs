mod env;
mod merge;
#[cfg(feature = "validate")]
mod validate;

pub use env::*;
pub use merge::*;
#[cfg(feature = "validate")]
pub use validate::*;

use crate::config::{ConfigError, HandlerError, MergeError, MergeResult, PartialConfig};
use schematic_types::Schema;
use std::str::FromStr;

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
