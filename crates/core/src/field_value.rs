use crate::args::NestedArg;
use crate::field::FieldArgs;
use crate::utils::ImplResult;
use crate::value::{Layer, Value};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::ops::Deref;
use syn::{Expr, Lit, Type};

#[derive(Debug)]
pub struct FieldValue(Value);

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
        if self.is_outer_option_wrapped() {
            return ImplResult::skipped();
        };

        let mut res = ImplResult::default();
        let mut wrap_with_some = false;

        // Extract the inner value first
        let mut value = if let Some(nested_ident) = &self.nested_ident {
            if field_args.default.is_some() {
                panic!("Cannot use `default` with `nested`.");
            }

            let ident = format_ident!("Partial{}", nested_ident);

            // quote! {
            //     <#nested_ident as schematic::PartialConfig>::default_values(context)?
            // }

            quote! {
                #ident::default_values(context)?
            }
        } else if let Some(expr) = &field_args.default {
            let ty = self.get_inner_type();

            match expr {
                Expr::Array(_) | Expr::Call(_) | Expr::Macro(_) | Expr::Tuple(_) => {
                    wrap_with_some = true;

                    quote! { #expr }
                }
                Expr::Path(func) => {
                    res.requires_internal = true;

                    quote! { handle_default_result(#func(context))? }
                }
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Str(string) => {
                        res.requires_internal = true;

                        quote! {
                            handle_default_result(#ty::try_from(#string))?
                        }
                    }
                    other => {
                        wrap_with_some = true;

                        quote! { #other }
                    }
                },
                invalid => {
                    panic!(
                        "Unsupported default value ({invalid:?}). May only provide literals, primitives, arrays, or tuples."
                    );
                }
            }
        } else {
            wrap_with_some = true;

            quote! {
                Default::default()
            }
        };

        // Then wrap with each layer
        if !self.layers.is_empty() {
            wrap_with_some = true;

            for layer in self.layers.iter().rev() {
                value = match layer {
                    Layer::Arc => quote! { Arc::new(#value) },
                    Layer::Box => quote! { Box::new(#value) },
                    Layer::Option => quote! { Some(#value) },
                    Layer::Rc => quote! { Rc::new(#value) },
                    Layer::Map(name)
                    | Layer::Set(name)
                    | Layer::Vec(name)
                    | Layer::Unknown(name) => {
                        let collection = format_ident!("{name}");

                        quote! { #collection::default() }
                    }
                };
            }
        }

        if wrap_with_some {
            value = quote! { Some(#value) };
        }

        res.value = value;
        res
    }

    #[cfg(not(feature = "env"))]
    pub fn impl_partial_env_value(&self, _field_args: &FieldArgs, _env_key: &str) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "env")]
    pub fn impl_partial_env_value(&self, field_args: &FieldArgs, env_key: &str) -> ImplResult {
        let mut res = ImplResult::default();

        if self.is_collection() {
            panic!("Collection types cannot be used with `env`.");
        } else if !self.layers.is_empty() {
            panic!("Wrapper types cannot be used with `env`.");
        }

        res.value = if let Some(nested_ident) = &self.nested_ident {
            let ident = format_ident!("Partial{}", nested_ident);

            if let Some(env_prefix) = &field_args.env_prefix {
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
            }
        } else if let Some(parse_env) = &field_args.parse_env {
            quote! {
                env.get_and_parse(#env_key, #parse_env)?
            }
        } else {
            quote! {
                env.get(#env_key)?
            }
        };

        res
    }

    #[cfg(not(feature = "extends"))]
    pub fn impl_partial_extends_from(
        &self,
        _field_args: &FieldArgs,
        _field_name: TokenStream,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "extends")]
    pub fn impl_partial_extends_from(
        &self,
        _field_args: &FieldArgs,
        field_name: TokenStream,
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
        field_name: TokenStream,
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
                if self.nested {
                    if self.is_collection() {
                        panic!("Collections with nested configs must manually define `merge`.");
                    }

                    quote! {
                        .nested(
                            &mut self.#field_name,
                            next.#field_name,
                        )?
                    }
                } else {
                    quote! {
                        .apply(
                            &mut self.#field_name,
                            next.#field_name,
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
}
