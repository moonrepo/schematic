use crate::config::{
    ConfigError, HandlerError, MergeError, MergeResult, ParseEnvResult, PartialConfig,
};
use schematic_types::Schema;
use std::str::FromStr;

// DEFAULT VALUES

pub fn handle_default_result<T, E: std::error::Error>(
    result: Result<T, E>,
) -> Result<T, ConfigError> {
    result.map_err(|error| ConfigError::InvalidDefaultValue(error.to_string()))
}

// ENV VARS

pub struct EnvManager {
    count: u8,
    prefix: String,
}

impl EnvManager {
    pub fn new<T: AsRef<str>>(prefix: Option<T>) -> Self {
        Self {
            count: 0,
            prefix: prefix
                .map(|pre| pre.as_ref().to_string())
                .unwrap_or_default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get<T: FromStr>(&mut self, key: &str) -> ParseEnvResult<T> {
        self.get_and_parse(key, |value| parse_value(value).map(|v| Some(v)))
    }

    pub fn get_and_parse<T>(
        &mut self,
        key: &str,
        parser: impl Fn(String) -> ParseEnvResult<T>,
    ) -> ParseEnvResult<T> {
        let key = format!("{}{key}", self.prefix);

        if let Ok(value) = std::env::var(&key) {
            return parser(value)
                .inspect(|inner| {
                    if inner.is_some() {
                        self.count += 1;
                    }
                })
                .map_err(|error| {
                    HandlerError(format!("Invalid environment variable {key}: {error}"))
                });
        }

        Ok(None)
    }

    pub fn nested<T>(&mut self, partial: Option<T>) -> ParseEnvResult<T> {
        if partial.is_some() {
            self.count += 1;
        }

        Ok(partial)
    }
}

// MERGING

pub struct MergeManager<'a, Ctx> {
    context: &'a Ctx,
}

impl<'a, Ctx> MergeManager<'a, Ctx> {
    pub fn new(context: &'a Ctx) -> Self {
        Self { context }
    }

    pub fn apply<T>(self, prev: &mut Option<T>, next: Option<T>) -> Result<Self, MergeError> {
        self.apply_with(prev, next, super::merge::replace)
    }

    pub fn apply_with<T>(
        self,
        prev: &mut Option<T>,
        next: Option<T>,
        merger: impl Fn(T, T, &Ctx) -> MergeResult<T>,
    ) -> Result<Self, MergeError> {
        let value = match (prev.take(), next) {
            (Some(prev), Some(next)) => merger(prev, next, self.context)?,
            (None, Some(next)) => Some(next),
            (other, None) => other,
        };

        if let Some(value) = value {
            prev.replace(value);
        }

        Ok(self)
    }

    pub fn nested<T: PartialConfig<Context = Ctx>>(
        self,
        prev: &mut Option<T>,
        next: Option<T>,
    ) -> Result<Self, MergeError> {
        self.apply_with(prev, next, |mut p, n, ctx| {
            p.merge(ctx, n)
                .map_err(|error| MergeError(error.to_string()))?;

            Ok(Some(p))
        })
    }
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
        _ => {}
    };
}
