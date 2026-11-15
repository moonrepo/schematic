use crate::config::{MergeError, MergeResult, PartialConfig};

pub struct MergeManager<'a, Ctx> {
    context: &'a Ctx,
}

impl<'a, Ctx> MergeManager<'a, Ctx> {
    pub fn new(context: &'a Ctx) -> Self {
        Self { context }
    }

    pub fn apply<T>(self, prev: &mut Option<T>, next: Option<T>) -> Result<Self, MergeError> {
        self.apply_with(prev, next, crate::merge::replace)
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

    /// Merge a nested config with a context of its own, from
    /// `#[setting(nested, context)]`.
    pub fn nested_with<T: PartialConfig>(
        self,
        prev: &mut Option<T>,
        next: Option<T>,
        context: &T::Context,
    ) -> Result<Self, MergeError> {
        match (prev.as_mut(), next) {
            (Some(prev), Some(next)) => {
                prev.merge(context, next)
                    .map_err(|error| MergeError(error.to_string()))?;
            }
            (None, Some(next)) => {
                prev.replace(next);
            }
            _ => {}
        };

        Ok(self)
    }

    /// Merge a nested config that holds its own type, with a context of its own.
    pub fn nested_boxed_with<T: PartialConfig>(
        self,
        prev: &mut Option<Box<T>>,
        next: Option<Box<T>>,
        context: &T::Context,
    ) -> Result<Self, MergeError> {
        match (prev.as_mut(), next) {
            (Some(prev), Some(next)) => {
                prev.merge(context, *next)
                    .map_err(|error| MergeError(error.to_string()))?;
            }
            (None, Some(next)) => {
                prev.replace(next);
            }
            _ => {}
        };

        Ok(self)
    }

    /// Merge a nested config that holds its own type, which is kept
    /// in a `Box` within the partial.
    pub fn nested_boxed<T: PartialConfig<Context = Ctx>>(
        self,
        prev: &mut Option<Box<T>>,
        next: Option<Box<T>>,
    ) -> Result<Self, MergeError> {
        self.apply_with(prev, next, |mut p, n, ctx| {
            p.merge(ctx, *n)
                .map_err(|error| MergeError(error.to_string()))?;

            Ok(Some(p))
        })
    }
}
