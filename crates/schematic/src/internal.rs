use crate::config::{ConfigError, PartialConfig};
use crate::merge::merge_partial;
use schematic_types::Schema;
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
        Ok(next)
    } else {
        Ok(prev)
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
                    field.nullify();

                    partialize_schema(field, true);
                }
            } else {
                for field in inner.fields.values_mut() {
                    partialize_schema(field, false);
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
        _ => {}
    };
}
