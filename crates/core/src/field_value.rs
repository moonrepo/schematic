use crate::args::NestedArg;
use crate::field::FieldArgs;
use crate::utils::{ImplResult, to_type_string};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, GenericArgument, Ident, Lit, PathArguments, PathSegment, Type};

#[derive(Debug, PartialEq)]
pub enum Layer {
    Arc,
    Box,
    Option,
    Rc,
    // Collections
    Map(String),
    Set(String),
    Vec(String),
    Unknown(String),
}

#[derive(Debug)]
pub struct FieldValue {
    pub inner_ty: Option<Type>,
    pub layers: Vec<Layer>,
    pub nested: bool,
    pub nested_ident: Option<Ident>,
    pub ty: Type,
    pub ty_string: String,
}

impl FieldValue {
    pub fn new(ty: Type, nested_arg: Option<&NestedArg>) -> Self {
        let mut nested = false;
        let mut nested_ident = None;
        let ty_string = to_type_string(ty.to_token_stream());

        // Determine nested state
        if let Some(nested_arg) = nested_arg {
            match nested_arg {
                NestedArg::Detect(state) => {
                    nested = *state;
                }
                NestedArg::Ident(ident) => {
                    nested = true;
                    nested_ident = Some(ident.to_owned());

                    if !ty_string.contains(&ident.to_string()) {
                        panic!(
                            "Nested configuration identifier `{ident}` does not exist within `{ty_string}`."
                        )
                    }
                }
            };
        }

        let mut value = FieldValue {
            inner_ty: None,
            nested,
            nested_ident,
            layers: vec![],
            ty_string,
            ty,
        };
        value.extract_type_information();
        value
    }

    pub fn extract_type_information(&mut self) {
        extract_type_information(&self.ty, &mut self.layers, |ty, segment| {
            self.inner_ty = Some(ty.to_owned());

            if self.nested && self.nested_ident.is_none() {
                self.nested_ident = Some(segment.ident.clone());
            }
        });

        if self.nested && self.nested_ident.is_none() {
            panic!(
                "Unable to extract the nested configuration identifier from `{}`. Try explicitly passing the identifier with `nested = ConfigName`.",
                self.ty_string
            )
        }
    }

    pub fn get_inner_type(&self) -> &Type {
        self.inner_ty.as_ref().unwrap_or(&self.ty)
    }

    pub fn is_collection(&self) -> bool {
        self.layers.iter().any(|layer| {
            matches!(
                layer,
                Layer::Map(_) | Layer::Set(_) | Layer::Vec(_) | Layer::Unknown(_)
            )
        })
    }

    pub fn is_outer_option_wrapped(&self) -> bool {
        self.layers
            .first()
            .is_some_and(|layer| *layer == Layer::Option)
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

fn extract_type_information(
    ty: &Type,
    layers: &mut Vec<Layer>,
    mut on_last: impl FnMut(&Type, &PathSegment),
) {
    // We don't need to traverse other types, just paths
    let Type::Path(ty_path) = ty else {
        return;
    };

    // Extract the last segment of the path, for example `Option`,
    // instead of the full path `std::option::Option`
    let last_segment = ty_path.path.segments.last().unwrap();

    match &last_segment.arguments {
        // We've reached the final segment
        PathArguments::None => {
            on_last(ty, last_segment);
        }

        // Attempt to drill deeper down
        PathArguments::AngleBracketed(args) => {
            extract_layer(last_segment, layers);

            if let Some(GenericArgument::Type(inner_ty)) = args.args.last() {
                extract_type_information(inner_ty, layers, on_last);
            }
        }

        // What to do here, anything?
        PathArguments::Parenthesized(_) => {}
    };
}

fn extract_layer(last_segment: &PathSegment, layers: &mut Vec<Layer>) {
    let layer = if last_segment.ident == "Option" {
        Layer::Option
    } else if last_segment.ident == "Arc" {
        Layer::Arc
    } else if last_segment.ident == "Box" {
        Layer::Box
    } else if last_segment.ident == "Rc" {
        Layer::Rc
    } else {
        let ident = last_segment.ident.to_string();

        if ident.ends_with("Vec") {
            Layer::Vec(ident)
        } else if ident.ends_with("Set") {
            Layer::Set(ident)
        } else if ident.ends_with("Map") {
            Layer::Map(ident)
        } else {
            Layer::Unknown(ident)
        }
    };

    layers.push(layer);
}
