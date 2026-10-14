use crate::config::{PartialConfig, Path, ValidateError, ValidateResult};

pub struct ValidateManager<'a, Ctx> {
    context: &'a Ctx,
    finalizing: bool,
    path: Path,

    pub errors: Vec<ValidateError>,
}

impl<'a, Ctx> ValidateManager<'a, Ctx> {
    pub fn new(context: &'a Ctx, finalizing: bool, path: Path) -> Self {
        Self {
            context,
            errors: vec![],
            finalizing,
            path,
        }
    }

    pub fn check<V, D>(
        &mut self,
        key: &str,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> ValidateResult,
    ) {
        self.check_at(self.path.join_key(key), value, data, validator);
    }

    pub fn check_variant<V, D>(
        &mut self,
        variant: &str,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> ValidateResult,
    ) {
        self.check_at(self.path.join_variant(variant), value, data, validator);
    }

    // The value is taken as-is rather than by reference, because a variant of
    // several values passes a tuple of references, not a reference to a tuple.
    //
    // The validator is an opaque callable rather than a `Validator` box, so
    // that the generated closure can deref coerce the value on its way in.
    // A `String` setting reaches a `&str` validator, a `Vec<T>` a `&[T]` one.
    fn check_at<V, D>(
        &mut self,
        path: Path,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> ValidateResult,
    ) {
        if let Err(error) = validator(value, data, self.context, self.finalizing) {
            self.errors.push(error.prepend_path(path));
        }
    }

    pub fn required(&mut self, key: &str) {
        self.required_at(self.path.join_key(key));
    }

    pub fn required_variant(&mut self, variant: &str) {
        self.required_at(self.path.join_variant(variant));
    }

    fn required_at(&mut self, path: Path) {
        if self.finalizing {
            self.errors
                .push(ValidateError::required().prepend_path(path));
        }
    }

    pub fn nested<S: PartialConfig<Context = Ctx>>(&mut self, key: &str, value: &S) {
        self.nested_at(self.path.join_key(key), value);
    }

    pub fn nested_variant<S: PartialConfig<Context = Ctx>>(
        &mut self,
        variant: &str,
        index: usize,
        value: &S,
    ) {
        self.nested_at(self.variant_path(variant, index), value);
    }

    /// The path of a setting, which a nested collection extends for each of
    /// its items.
    pub fn key_path(&self, key: &str) -> Path {
        self.path.join_key(key)
    }

    /// The path of a value within a variant, which a nested collection
    /// extends for each of its items.
    pub fn variant_path(&self, variant: &str, index: usize) -> Path {
        self.path.join_variant(variant).join_index(index)
    }

    /// Validate a nested configuration at the provided path, like an item
    /// within a nested collection.
    pub fn nested_at<S: PartialConfig<Context = Ctx>>(&mut self, path: Path, value: &S) {
        if let Err(errors) = value.validate_with_path(self.context, self.finalizing, path) {
            self.errors.extend(errors);
        }
    }
}
