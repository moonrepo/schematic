use crate::config::{ConfigError, PartialConfig};

/// A partial with its default values and environment variables applied, and
/// the layers that were, from [`apply_layers`].
pub struct AppliedLayers<T> {
    /// The default values merged under every other layer, when there were any.
    pub defaults: Option<T>,

    /// The environment variables merged over the sources, when any were set.
    pub env: Option<T>,

    /// The result, which still has to be finalized.
    pub partial: T,
}

/// Apply the default values and environment variables of a config to
/// `partial`, the merge of every source, or `None` when there were none.
/// This is what the loader does to a config, and finalizing does to each item
/// of a nested collection, which no layer holds.
///
/// The variables are read first, against the sources, as a tag's variable may
/// pick the variant whose defaults apply, and the defaults are read against
/// the result. They then go under everything else merged together, rather
/// than under the first source, so that a source picking one variant can't
/// drop the defaults of another that a later layer picks. A missing partial is
/// viewed as its [`Default`], but never merged, as an enum's would replace the
/// value its defaults hold.
pub fn apply_layers<T: PartialConfig>(
    partial: Option<T>,
    context: &T::Context,
) -> Result<AppliedLayers<T>, ConfigError> {
    #[cfg(feature = "env")]
    let (partial, env) = {
        let env = match &partial {
            Some(partial) => partial.env_values()?,
            None => T::default().env_values()?,
        };

        let partial = match (partial, &env) {
            (Some(mut partial), Some(env)) => {
                partial.merge(context, env.clone())?;

                Some(partial)
            }
            (None, Some(env)) => Some(env.clone()),
            (partial, None) => partial,
        };

        (partial, env)
    };

    #[cfg(not(feature = "env"))]
    let env = None;

    let defaults = match &partial {
        Some(partial) => partial.default_values(context)?,
        None => T::default().default_values(context)?,
    };

    let partial = match (defaults.clone(), partial) {
        (Some(mut defaults), Some(partial)) => {
            defaults.merge(context, partial)?;
            defaults
        }
        (Some(defaults), None) => defaults,
        (None, partial) => partial.unwrap_or_default(),
    };

    Ok(AppliedLayers {
        defaults,
        env,
        partial,
    })
}

/// The default values of a nested config, viewing one that's missing as its
/// [`Default`], as a parent's defaults include those of a config no source
/// provided.
pub fn nested_default_values<T: PartialConfig>(
    value: Option<&T>,
    context: &T::Context,
) -> Result<Option<T>, ConfigError> {
    match value {
        Some(value) => value.default_values(context),
        None => T::default().default_values(context),
    }
}

/// The environment variables of a nested config, viewing one that's missing
/// as its [`Default`], so that a variable can provide a config that no source
/// did. A parent's `#[setting(nested, env_prefix)]` is passed as `prefix`.
#[cfg(feature = "env")]
pub fn nested_env_values<T: PartialConfig>(
    value: Option<&T>,
    prefix: Option<&str>,
) -> Result<Option<T>, ConfigError> {
    let read = |value: &T| match prefix {
        Some(_) => value.env_values_with_prefix(prefix),
        None => value.env_values(),
    };

    match value {
        Some(value) => read(value),
        None => read(&T::default()),
    }
}

/// Finalize a partial whose default values and environment variables no layer
/// above it holds, like an item of a nested collection. They're applied to it
/// as the loader applies a config's, with [`apply_layers`].
pub fn finalize_with_layers<T: PartialConfig>(
    partial: T,
    context: &T::Context,
) -> Result<T, ConfigError> {
    apply_layers(Some(partial), context)?
        .partial
        .finalize(context)
}

/// Finalize a config within a tuple variant that holds several values, whose
/// environment variables no layer holds, as the enum's would have to restate
/// the values beside it. Its defaults are part of the enum's own.
#[cfg(feature = "env")]
pub fn finalize_with_env<T: PartialConfig>(
    mut partial: T,
    context: &T::Context,
) -> Result<T, ConfigError> {
    if let Some(env) = partial.env_values()? {
        partial.merge(context, env)?;
    }

    partial.finalize(context)
}

/// Finalize a config within a tuple variant that holds several values. Without
/// environment variables, there's nothing for a layer to have missed.
#[cfg(not(feature = "env"))]
pub fn finalize_with_env<T: PartialConfig>(
    partial: T,
    context: &T::Context,
) -> Result<T, ConfigError> {
    partial.finalize(context)
}
