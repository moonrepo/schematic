use crate::config::{ConfigError, IntoValidateErrors, PartialConfig, Path, ValidateError};

pub struct ValidateManager<'a, Ctx> {
    context: &'a Ctx,
    finalizing: bool,
    path: Path,
    errors: Vec<ValidateError>,
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

    /// Return every error the checks reported, if any did.
    pub fn finish(self) -> Result<(), Vec<ValidateError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    pub fn check<V, D, E: IntoValidateErrors>(
        &mut self,
        key: &str,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> Result<(), E>,
    ) {
        self.check_at(self.path.join_key(key), value, data, validator);
    }

    pub fn check_variant<V, D, E: IntoValidateErrors>(
        &mut self,
        variant: &str,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> Result<(), E>,
    ) {
        self.check_at(self.path.join_variant(variant), value, data, validator);
    }

    /// Validate the config as a whole, from `#[config(validate)]`, so its
    /// errors are reported at this path, unless they name settings within it.
    pub fn check_config<D, E: IntoValidateErrors>(
        &mut self,
        data: &D,
        validator: impl FnOnce(&D, &Ctx, bool) -> Result<(), E>,
    ) {
        if let Err(errors) = validator(data, self.context, self.finalizing) {
            self.push_errors(self.path.clone(), errors);
        }
    }

    // The value is taken as-is rather than by reference, because a variant of
    // several values passes a tuple of references, not a reference to a tuple.
    //
    // The validator is an opaque callable rather than a `Validator` box, so
    // that the generated closure can deref coerce the value on its way in.
    // A `String` setting reaches a `&str` validator, a `Vec<T>` a `&[T]` one.
    fn check_at<V, D, E: IntoValidateErrors>(
        &mut self,
        path: Path,
        value: V,
        data: &D,
        validator: impl FnOnce(V, &D, &Ctx, bool) -> Result<(), E>,
    ) {
        if let Err(errors) = validator(value, data, self.context, self.finalizing) {
            self.push_errors(path, errors);
        }
    }

    fn push_errors(&mut self, path: Path, errors: impl IntoValidateErrors) {
        self.errors.extend(
            errors
                .into_validate_errors()
                .into_iter()
                .map(|error| error.prepend_path(path.clone())),
        );
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

    /// Validate a nested config with a context of its own, from
    /// `#[setting(nested, context)]`.
    pub fn nested_with<S: PartialConfig>(&mut self, key: &str, value: &S, context: &S::Context) {
        self.nested_at_with(self.path.join_key(key), value, context);
    }

    /// Validate a nested configuration whose settings are flattened into
    /// this one, so its settings are reported at this path, as written.
    pub fn flattened<S: PartialConfig<Context = Ctx>>(&mut self, value: &S) {
        self.nested_at(self.path.clone(), value);
    }

    /// Validate a flattened config with a context of its own.
    pub fn flattened_with<S: PartialConfig>(&mut self, value: &S, context: &S::Context) {
        self.nested_at_with(self.path.clone(), value, context);
    }

    pub fn nested_variant<S: PartialConfig<Context = Ctx>>(
        &mut self,
        variant: &str,
        index: usize,
        value: &S,
    ) {
        self.nested_at(self.variant_path(variant, index), value);
    }

    /// Validate a nested config within a variant, with a context of its own.
    pub fn nested_variant_with<S: PartialConfig>(
        &mut self,
        variant: &str,
        index: usize,
        value: &S,
        context: &S::Context,
    ) {
        self.nested_at_with(self.variant_path(variant, index), value, context);
    }

    /// Validate the settings of a named variant, which are reported under the
    /// variant, like `Name.setting`, as a struct's are under the struct.
    pub fn named_variant(&mut self, variant: &str, validate: impl FnOnce(&mut Self)) {
        let path = self.path.join_variant(variant);
        let parent = std::mem::replace(&mut self.path, path);

        validate(self);

        self.path = parent;
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
        self.nested_at_with(path, value, self.context);
    }

    /// Validate a nested configuration at the provided path, with a
    /// context of its own.
    ///
    /// This goes through the public `validate`, which a hand-written partial
    /// overrides, rather than the hidden `validate_with_path` that a derived
    /// one does, so that either is validated when nested. Its errors are then
    /// moved under the path, and any other error is reported at the path.
    pub fn nested_at_with<S: PartialConfig>(
        &mut self,
        path: Path,
        value: &S,
        context: &S::Context,
    ) {
        match value.validate(context, self.finalizing) {
            Ok(()) => {}
            Err(ConfigError::Validator { error, .. }) => {
                self.errors.extend(
                    error
                        .errors
                        .into_iter()
                        .map(|error| error.prepend_path(path.clone())),
                );
            }
            Err(error) => {
                self.errors
                    .push(ValidateError::new(error.to_string()).prepend_path(path));
            }
        }
    }
}
