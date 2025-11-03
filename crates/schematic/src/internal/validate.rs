use crate::config::{PartialConfig, Path, ValidateError, Validator};

pub struct ValidateManager<'a, Ctx> {
    context: &'a Ctx,
    finalize: bool,
    path: Path,

    pub errors: Vec<ValidateError>,
}

impl<'a, Ctx> ValidateManager<'a, Ctx> {
    pub fn new(context: &'a Ctx, finalize: bool, path: Path) -> Self {
        Self {
            context,
            errors: vec![],
            finalize,
            path,
        }
    }

    pub fn check<V, D>(&mut self, key: &str, value: &V, data: &D, validator: Validator<V, D, Ctx>) {
        if let Err(error) = validator(value, data, self.context, self.finalize) {
            self.errors
                .push(error.prepend_path(self.path.join_key(key)));
        }
    }

    pub fn nested<S: PartialConfig<Context = Ctx>>(&mut self, key: &str, value: &S) {
        if let Err(errors) =
            value.validate_with_path(self.context, self.finalize, self.path.join_key(key))
        {
            self.errors.extend(errors);
        }
    }

    pub fn nested_list<'v, I: IntoIterator<Item = &'v S>, S: PartialConfig<Context = Ctx> + 'v>(
        &mut self,
        key: &str,
        list: I,
    ) {
        for (i, item) in list.into_iter().enumerate() {
            if let Err(errors) = item.validate_with_path(
                self.context,
                self.finalize,
                self.path.join_key(key).join_index(i),
            ) {
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
        for (sub_key, value) in map.into_iter() {
            if let Err(errors) = value.validate_with_path(
                self.context,
                self.finalize,
                self.path.join_key(key).join_key(sub_key),
            ) {
                self.errors.extend(errors);
            }
        }
    }
}
