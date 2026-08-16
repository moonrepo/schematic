use crate::config::{PartialConfig, Path, ValidateError, Validator};

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

    pub fn check<V, D>(&mut self, key: &str, value: &V, data: &D, validator: Validator<V, D, Ctx>) {
        self.check_at(self.path.join_key(key), value, data, validator);
    }

    pub fn check_variant<V, D>(
        &mut self,
        variant: &str,
        value: &V,
        data: &D,
        validator: Validator<V, D, Ctx>,
    ) {
        self.check_at(self.path.join_variant(variant), value, data, validator);
    }

    fn check_at<V, D>(&mut self, path: Path, value: &V, data: &D, validator: Validator<V, D, Ctx>) {
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

    fn nested_at<S: PartialConfig<Context = Ctx>>(&mut self, path: Path, value: &S) {
        if let Err(errors) = value.validate_with_path(self.context, self.finalizing, path) {
            self.errors.extend(errors);
        }
    }

    fn variant_path(&self, variant: &str, index: usize) -> Path {
        self.path.join_variant(variant).join_index(index)
    }

    pub fn nested_list<'v, I: IntoIterator<Item = &'v S>, S: PartialConfig<Context = Ctx> + 'v>(
        &mut self,
        key: &str,
        list: I,
    ) {
        self.nested_list_at(self.path.join_key(key), list);
    }

    pub fn nested_variant_list<
        'v,
        I: IntoIterator<Item = &'v S>,
        S: PartialConfig<Context = Ctx> + 'v,
    >(
        &mut self,
        variant: &str,
        index: usize,
        list: I,
    ) {
        self.nested_list_at(self.variant_path(variant, index), list);
    }

    fn nested_list_at<'v, I: IntoIterator<Item = &'v S>, S: PartialConfig<Context = Ctx> + 'v>(
        &mut self,
        path: Path,
        list: I,
    ) {
        for (i, item) in list.into_iter().enumerate() {
            if let Err(errors) =
                item.validate_with_path(self.context, self.finalizing, path.join_index(i))
            {
                self.errors.extend(errors);
            }
        }
    }

    pub fn nested_map<
        'v,
        I: IntoIterator<Item = (&'v String, &'v S)>,
        S: PartialConfig<Context = Ctx> + 'v,
    >(
        &mut self,
        key: &str,
        map: I,
    ) {
        self.nested_map_at(self.path.join_key(key), map);
    }

    pub fn nested_variant_map<
        'v,
        I: IntoIterator<Item = (&'v String, &'v S)>,
        S: PartialConfig<Context = Ctx> + 'v,
    >(
        &mut self,
        variant: &str,
        index: usize,
        map: I,
    ) {
        self.nested_map_at(self.variant_path(variant, index), map);
    }

    fn nested_map_at<
        'v,
        I: IntoIterator<Item = (&'v String, &'v S)>,
        S: PartialConfig<Context = Ctx> + 'v,
    >(
        &mut self,
        path: Path,
        map: I,
    ) {
        for (sub_key, value) in map.into_iter() {
            if let Err(errors) =
                value.validate_with_path(self.context, self.finalizing, path.join_key(sub_key))
            {
                self.errors.extend(errors);
            }
        }
    }
}
