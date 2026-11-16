use crate::args::NestedArg;
use crate::field::{EnvKey, FieldArgs};
use crate::utils::ImplResult;
use crate::value::{Layer, Value, path_names_a_value, wrap_layer};
use proc_macro2::TokenStream;
use quote::quote;
use std::ops::Deref;
use syn::{Expr, ExprPath, Type};

#[derive(Debug)]
pub struct FieldValue(Value);

fn wrap_default_layers(layers: &[&Layer], mut value: TokenStream) -> TokenStream {
    for layer in layers.iter().rev() {
        // Wrappers that were not stripped, like `Box<str>`, provide a default
        // for the entire value, as the inner value may not have one
        if layer.is_wrapper() {
            continue;
        }

        value = wrap_layer(layer, value);
    }

    value
}

impl Deref for FieldValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FieldValue {
    pub fn new(
        ty: Type,
        nested_arg: Option<&NestedArg>,
        self_ty: &Type,
        context_fn: Option<ExprPath>,
    ) -> Self {
        FieldValue(Value::new(ty, nested_arg, self_ty).with_context(context_fn))
    }

    /// Generate the default value of a setting, for `default_values`, where
    /// `access` is the setting within the partial its defaults go under.
    pub fn impl_partial_default_value(
        &self,
        field_args: &FieldArgs,
        access: &TokenStream,
    ) -> ImplResult {
        let outer_option = self.is_outer_option_wrapped();

        // Nested configs source their defaults from the inner partial, viewed
        // as it is beneath them, so an enum's are those of its variant
        if self.nested && !self.is_collection() {
            return self.impl_partial_default_value_nested(field_args, access);
        }

        // Optional settings default to `None`, unless a value is provided,
        // and required settings have no default at all, so that the partial
        // stays empty until a layer provides a value
        if (outer_option && field_args.default.is_none()) || field_args.required {
            return ImplResult::skipped();
        };

        let mut res = ImplResult::default();

        // Wrappers are stripped from the partial, so only the structural
        // layers need to be applied. The outermost `Option` is the partial's
        // own, and is applied last.
        let mut layers = self.get_partial_layers();

        if outer_option {
            layers.remove(0);
        }

        // Collections of nested configs start empty, as each item has its
        // defaults applied when it's finalized
        if self.nested {
            if field_args.default.is_some() {
                panic!("Cannot use `default` with `nested`.");
            }

            let value = wrap_default_layers(&layers, quote! { Default::default() });

            res.value = quote! { Some(#value) };

            return res;
        }

        match field_args.default.as_ref() {
            // Handler functions return the entire value
            Some(Expr::Path(func)) if !path_names_a_value(&func.path) => {
                res.value = quote! { schematic::internal::defaults::handle_default_result(#func(context))? };
            }
            Some(expr) => {
                let value = self.impl_default_value(expr, &layers);

                res.value = quote! { Some(#value) };
            }
            // Otherwise fallback to the type default
            None => {
                let value = wrap_default_layers(&layers, quote! { Default::default() });

                res.value = quote! { Some(#value) };
            }
        };

        res
    }

    /// Generate the defaults of a nested config that isn't in a collection.
    /// A bare one that's missing is viewed as its default, as the full type
    /// always holds one, while an optional one only has defaults when it's
    /// present, and a recursive one would otherwise never end.
    fn impl_partial_default_value_nested(
        &self,
        field_args: &FieldArgs,
        access: &TokenStream,
    ) -> ImplResult {
        if field_args.default.is_some() {
            panic!("Cannot use `default` with `nested`.");
        }

        let partial = self.get_nested_partial_type();
        let context = self.get_nested_context();

        let value = if !self.is_outer_option_wrapped() {
            quote! { schematic::internal::layers::nested_default_values(#access.as_ref(), #context)? }
        } else if self.keep_box {
            quote! {
                match #access.as_deref() {
                    Some(inner) => <#partial as schematic::PartialConfig>::default_values(inner, #context)?.map(Box::new),
                    None => None,
                }
            }
        } else {
            quote! {
                match #access.as_ref() {
                    Some(inner) => <#partial as schematic::PartialConfig>::default_values(inner, #context)?,
                    None => None,
                }
            }
        };

        ImplResult {
            value,
            ..Default::default()
        }
    }

    #[cfg(not(feature = "env"))]
    pub fn impl_partial_env_value(
        &self,
        _field_args: &FieldArgs,
        _env_key: Option<&EnvKey>,
        _access: &TokenStream,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    /// Generate the value of a setting from the environment, for `env_values`,
    /// where `access` is the setting within the partial the variables go over.
    #[cfg(feature = "env")]
    pub fn impl_partial_env_value(
        &self,
        field_args: &FieldArgs,
        env_key: Option<&EnvKey>,
        access: &TokenStream,
    ) -> ImplResult {
        let mut res = ImplResult::default();

        // Values can only be sourced from the environment when the type
        // is bare or wrapped in a single `Option`, as other layers and
        // collections cannot be parsed from a string. Unless a `parse_env`
        // function is provided, which handles the conversion itself.
        let supported = self.supports_env_value(field_args.parse_env.is_some());

        // A nested config is viewed as it is beneath the variables, so an
        // enum's are those of its variant, and a missing one as its default,
        // so that a variable can provide it
        if self.nested {
            if !supported {
                if field_args.env_prefix.is_some() {
                    panic!("Cannot use `env_prefix` with collections or wrapped types.");
                }

                return ImplResult::skipped();
            }

            let prefix = if let Some(env_prefix) = &field_args.env_prefix {
                if env_prefix.is_empty() {
                    panic!("Attribute `env_prefix` cannot be empty.");
                }

                quote! { Some(#env_prefix) }
            } else {
                quote! { None }
            };

            res.value = quote! {
                env.nested(schematic::internal::layers::nested_env_values(#access.as_ref(), #prefix)?)?
            };

            return res;
        }

        if !supported {
            // Only error for explicit `env` keys, and silently skip keys
            // derived from the container-level `env_prefix`
            if field_args.env.is_some() {
                if self.is_collection() {
                    panic!("Collection types cannot be used with `env`.");
                } else {
                    panic!("Wrapper types cannot be used with `env`.");
                }
            }

            return ImplResult::skipped();
        }

        let Some(env_key) = env_key else {
            return ImplResult::skipped();
        };

        // Explicit keys are read as-is and take precedence over any prefix,
        // while derived keys have the prefix applied at runtime
        let (key, get, get_and_parse) = match env_key {
            EnvKey::Explicit(key) => (key, quote! { get }, quote! { get_and_parse }),
            EnvKey::Derived(key) => (
                key,
                quote! { get_prefixed },
                quote! { get_and_parse_prefixed },
            ),
        };

        res.value = if let Some(parse_env) = &field_args.parse_env {
            quote! {
                env.#get_and_parse(#key, #parse_env)?
            }
        } else {
            quote! {
                env.#get(#key)?
            }
        };

        res
    }

    #[cfg(not(feature = "extends"))]
    pub fn impl_partial_extends_from(
        &self,
        _field_args: &FieldArgs,
        _field_name: &TokenStream,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "extends")]
    pub fn impl_partial_extends_from(
        &self,
        _field_args: &FieldArgs,
        field_name: &TokenStream,
    ) -> ImplResult {
        let value = match self.ty_string.as_str() {
            "String" | "Option<String>" => {
                quote! {
                    self.#field_name
                        .as_ref()
                        .map(|inner| schematic::ExtendsFrom::String(inner.to_owned()))
                }
            }
            "Vec<String>" | "Option<Vec<String>>" => {
                quote! {
                    self.#field_name
                        .as_ref()
                        .map(|inner| schematic::ExtendsFrom::List(inner.to_owned()))
                }
            }
            "ExtendsFrom"
            | "schematic::ExtendsFrom"
            | "Option<ExtendsFrom>"
            | "Option<schematic::ExtendsFrom>" => {
                quote! {
                    self.#field_name.clone()
                }
            }
            inner => {
                panic!(
                    "Only `String`, `Vec<String>`, or `schematic::ExtendsFrom` are supported when using `extend` for {field_name}. Received `{inner}`."
                );
            }
        };

        ImplResult {
            value,
            ..Default::default()
        }
    }

    /// Generate a merge of `next` into `prev`, as a method of a `MergeManager`
    /// chain. The `prev` is a mutable reference to the setting's `Option`, and
    /// `next` the `Option` itself.
    pub fn impl_partial_merge(
        &self,
        field_args: &FieldArgs,
        prev: &TokenStream,
        next: &TokenStream,
    ) -> ImplResult {
        let value = match field_args.merge.as_ref() {
            Some(func) => {
                if self.nested && !self.is_collection() {
                    panic!("Nested configs do not support `merge` unless wrapped in a collection.");
                }

                quote! {
                    .apply_with(
                        #prev,
                        #next,
                        #func,
                    )?
                }
            }
            _ => {
                // Nested configs are merged recursively, but collections of
                // them are replaced, as there's no way to know how to pair
                // up their items. Define `merge` to customize this.
                if self.nested && !self.is_collection() {
                    // The partial field is always wrapped in an `Option`
                    return self.impl_partial_merge_nested(prev, next, true);
                }

                // A `MergeList` is merged as each source directs
                if self.is_merge_list() {
                    quote! {
                        .apply_with(
                            #prev,
                            #next,
                            schematic::merge::merge_list,
                        )?
                    }
                } else {
                    quote! {
                        .apply(
                            #prev,
                            #next,
                        )?
                    }
                }
            }
        };

        ImplResult {
            value,
            ..Default::default()
        }
    }

    #[cfg(not(feature = "validate"))]
    pub fn impl_partial_validate(&self, _field_args: &FieldArgs, _field_name: &str) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "validate")]
    pub fn impl_partial_validate(&self, field_args: &FieldArgs, field_name: &str) -> ImplResult {
        let mut res = ImplResult::default();

        if let Some(expr) = field_args.validate.as_deref() {
            let func = match expr {
                // func(arg)() - returns a validator
                // func() - is the validator itself
                Expr::Call(_) | Expr::Path(_) => quote! { #expr },
                _ => {
                    panic!("Unsupported `validate` syntax.");
                }
            };

            // Called through a closure rather than passed by value, so that
            // the setting deref coerces on its way in, letting a `String`
            // reach a `&str` validator and a `Vec<T>` a `&[T]` one
            res.value = quote! {
                validate.check(#field_name, setting, self, |v, d, c, f| #func(v, d, c, f));
            };
        } else {
            res.no_value = true;
        }

        res
    }
}
