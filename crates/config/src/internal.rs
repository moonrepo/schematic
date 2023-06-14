use crate::config::PartialConfig;
use crate::errors::ConfigError;
use crate::merge::merge_partial;
use schematic_types::SchemaType;
use std::{env, str::FromStr};

pub fn default_from_env_var<T: FromStr>(key: &str) -> Result<Option<T>, ConfigError> {
    parse_from_env_var(key, |var| parse_value(var).map(|v| Some(v)))
}

pub fn parse_from_env_var<T>(
    key: &str,
    parser: impl Fn(String) -> Result<Option<T>, ConfigError>,
) -> Result<Option<T>, ConfigError> {
    if let Ok(var) = env::var(key) {
        let value =
            parser(var).map_err(|e| ConfigError::InvalidEnvVar(key.to_owned(), e.to_string()))?;

        return Ok(value);
    }

    Ok(None)
}

pub fn parse_value<T: FromStr, V: AsRef<str>>(value: V) -> Result<T, ConfigError> {
    let value = value.as_ref();

    value.parse::<T>().map_err(|_| {
        ConfigError::Message(format!(
            "Failed to parse \"{value}\" into the correct type."
        ))
    })
}

#[allow(clippy::unnecessary_unwrap)]
pub fn merge_setting<T, C>(
    prev: Option<T>,
    next: Option<T>,
    context: &C,
    merger: impl Fn(T, T, &C) -> Result<Option<T>, ConfigError>,
) -> Result<Option<T>, ConfigError> {
    if prev.is_some() && next.is_some() {
        merger(prev.unwrap(), next.unwrap(), context)
    } else if next.is_some() {
        Ok(next)
    } else {
        Ok(prev)
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn merge_partial_setting<T: PartialConfig>(
    prev: Option<T>,
    next: Option<T>,
    context: &T::Context,
) -> Result<Option<T>, ConfigError> {
    if prev.is_some() && next.is_some() {
        merge_partial(prev.unwrap(), next.unwrap(), context)
    } else if next.is_some() {
        // For optional nested configs, defaults values are not provided by the
        // `default_values` method since we return `None`. However, if a value
        // is provided during the merging process, then we need to lazily load
        // the default values to merge against.
        merge_partial(T::default_values(context)?, next.unwrap(), context)
    } else {
        Ok(prev)
    }
}

pub fn partialize_schema(schema: &mut SchemaType) {
    use schematic_types::*;

    match schema {
        SchemaType::Array(ArrayType { items_type, .. }) => {
            partialize_schema(items_type);
        }
        SchemaType::Object(ObjectType {
            key_type,
            value_type,
            ..
        }) => {
            partialize_schema(key_type);
            partialize_schema(value_type);
        }
        SchemaType::Struct(inner) => {
            if let Some(name) = &inner.name {
                inner.name = Some(format!("Partial{}", name));
            }

            inner.partial = true;

            for field in inner.fields.iter_mut() {
                field.optional = true;
                field.nullable = true;

                partialize_schema(&mut field.type_of);

                field.type_of = SchemaType::nullable(field.type_of.clone());
            }
        }
        SchemaType::Tuple(TupleType { items_types, .. }) => {
            for item in items_types {
                partialize_schema(item);
            }
        }
        SchemaType::Union(UnionType {
            variants_types,
            variants,
            ..
        }) => {
            for variant in variants_types {
                partialize_schema(variant);
            }

            if let Some(fields) = variants {
                for field in fields.iter_mut() {
                    field.optional = true;
                    field.nullable = true;

                    partialize_schema(&mut field.type_of);
                }
            }
        }
        _ => {}
    };
}
