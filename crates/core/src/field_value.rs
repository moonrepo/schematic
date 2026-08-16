use crate::args::NestedArg;
use crate::field::{EnvKey, FieldArgs};
use crate::utils::ImplResult;
use crate::value::{Layer, Value};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{Expr, Lit, Type};

#[derive(Debug)]
pub struct FieldValue(Value);

fn wrap_layer(layer: &Layer, value: TokenStream) -> TokenStream {
    match layer {
        Layer::Arc => quote! { Arc::new(#value) },
        Layer::Box => quote! { Box::new(#value) },
        Layer::Option => quote! { Some(#value) },
        Layer::Rc => quote! { Rc::new(#value) },
        // Collections reset to empty, discarding the inner value
        Layer::Map(name) | Layer::Set(name) | Layer::Vec(name) | Layer::Unknown(name) => {
            let collection = format_ident!("{name}");

            quote! { #collection::default() }
        }
    }
}

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
    pub fn new(ty: Type, nested_arg: Option<&NestedArg>) -> Self {
        FieldValue(Value::new(ty, nested_arg))
    }

    pub fn impl_partial_default_value(&self, field_args: &FieldArgs) -> ImplResult {
        let outer_option = self.is_outer_option_wrapped();

        // Optional settings default to `None`, unless a value is provided
        if outer_option && field_args.default.is_none() {
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

        // Nested configs source their defaults from the inner partial
        if let Some(nested_ident) = &self.nested_ident {
            if field_args.default.is_some() {
                panic!("Cannot use `default` with `nested`.");
            }

            let ident = format_ident!("Partial{}", nested_ident);

            res.value = if self.is_collection() {
                // Collections of nested configs start empty
                let value = wrap_default_layers(&layers, quote! { Default::default() });

                quote! { Some(#value) }
            } else if layers.is_empty() {
                quote! { #ident::default_values(context)? }
            } else {
                // Wrap the inner partial with each layer
                let mut value = quote! { inner };

                for layer in layers.iter().rev() {
                    value = wrap_layer(layer, value);
                }

                quote! { #ident::default_values(context)?.map(|inner| #value) }
            };

            return res;
        }

        match field_args.default.as_ref() {
            // Handler functions return the entire value
            Some(Expr::Path(func)) => {
                res.requires_internal = true;
                res.value = quote! { handle_default_result(#func(context))? };
            }
            // Explicit defaults provide the value up to the outermost
            // collection, so only wrap with the layers outside of it
            Some(expr) => {
                let mut value = match expr {
                    Expr::Array(_) | Expr::Call(_) | Expr::Macro(_) | Expr::Tuple(_) => {
                        quote! { #expr }
                    }
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Str(string) => {
                            let ty = self.get_inner_type();
                            res.requires_internal = true;

                            quote! {
                                handle_default_result(#ty::try_from(#string))?
                            }
                        }
                        other => quote! { #other },
                    },
                    invalid => {
                        panic!(
                            "Unsupported default value ({invalid:?}). May only provide literals, primitives, arrays, or tuples."
                        );
                    }
                };

                let outer = layers
                    .iter()
                    .position(|layer| layer.is_collection())
                    .map(|index| &layers[..index])
                    .unwrap_or(&layers[..]);

                for layer in outer.iter().rev() {
                    value = wrap_layer(layer, value);
                }

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

    #[cfg(not(feature = "env"))]
    pub fn impl_partial_env_value(
        &self,
        _field_args: &FieldArgs,
        _env_key: Option<&EnvKey>,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "env")]
    pub fn impl_partial_env_value(
        &self,
        field_args: &FieldArgs,
        env_key: Option<&EnvKey>,
    ) -> ImplResult {
        let mut res = ImplResult::default();

        // Values can only be sourced from the environment when the type
        // is bare or wrapped in a single `Option`, as other layers and
        // collections cannot be parsed from a string. Unless a `parse_env`
        // function is provided, which handles the conversion itself.
        let layers = self.get_partial_layers();
        let supported = field_args.parse_env.is_some()
            || layers.is_empty()
            || (layers.len() == 1 && self.is_outer_option_wrapped());

        if let Some(nested_ident) = &self.nested_ident {
            if !supported {
                if field_args.env_prefix.is_some() {
                    panic!("Cannot use `env_prefix` with collections or wrapped types.");
                }

                return ImplResult::skipped();
            }

            let ident = format_ident!("Partial{}", nested_ident);

            res.value = if let Some(env_prefix) = &field_args.env_prefix {
                if env_prefix.is_empty() {
                    panic!("Attribute `env_prefix` cannot be empty.");
                }

                quote! {
                    env.nested(#ident::env_values_with_prefix(Some(#env_prefix))?)?
                }
            } else {
                quote! {
                    env.nested(#ident::env_values()?)?
                }
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

    pub fn impl_partial_merge(
        &self,
        field_args: &FieldArgs,
        field_name: &TokenStream,
    ) -> ImplResult {
        let value = match field_args.merge.as_ref() {
            Some(func) => {
                if self.nested && !self.is_collection() {
                    panic!("Nested configs do not support `merge` unless wrapped in a collection.");
                }

                quote! {
                    .apply_with(
                        &mut self.#field_name,
                        next.#field_name,
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
                    return self.impl_partial_merge_nested(
                        &quote! { &mut self.#field_name },
                        &quote! { next.#field_name },
                        true,
                    );
                }

                quote! {
                    .apply(
                        &mut self.#field_name,
                        next.#field_name,
                    )?
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
                // func(arg)() - already returns a boxed validator
                Expr::Call(func) => quote! { #func },
                // func() - must be boxed
                Expr::Path(func) => quote! { Box::new(#func) },
                _ => {
                    panic!("Unsupported `validate` syntax.");
                }
            };

            res.value = quote! {
                validate.check(#field_name, setting, self, #func);
            };
        } else {
            res.no_value = true;
        }

        res
    }
}
