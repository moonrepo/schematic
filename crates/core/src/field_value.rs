use crate::field::{FieldArgs, FieldNestedArg};
use crate::utils::to_type_string;
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
    pub layers: Vec<Layer>,
    pub nested: bool,
    pub nested_ident: Option<Ident>,
    pub ty: Type,
    pub ty_string: String,
}

impl FieldValue {
    pub fn new(ty: Type, nested_arg: Option<&FieldNestedArg>) -> Self {
        let mut nested = false;
        let mut nested_ident = None;
        let mut layers = vec![];
        let ty_string = to_type_string(ty.to_token_stream());

        // Determine nested state
        if let Some(nested_arg) = nested_arg {
            match nested_arg {
                FieldNestedArg::Detect(state) => {
                    nested = *state;
                }
                FieldNestedArg::Ident(ident) => {
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

        // Extract type information
        if let Some(custom_ident) =
            extract_type_information(&ty, &mut layers, nested && nested_ident.is_none())
        {
            nested_ident = Some(custom_ident);
        }

        if nested_ident.is_none() && nested {
            panic!(
                "Unable to extract the nested configuration identifier from `{ty_string}`. Try explicitly passing the identifier with `nested = ConfigName`."
            )
        }

        let value = Self {
            nested,
            nested_ident,
            layers,
            ty_string,
            ty,
        };

        // dbg!(&value);

        value
    }

    pub fn is_outer_option_wrapped(&self) -> bool {
        self.layers
            .first()
            .is_some_and(|wrapper| *wrapper == Layer::Option)
    }

    pub fn impl_partial_default_value(&self, field_args: &FieldArgs) -> Option<TokenStream> {
        if self.is_outer_option_wrapped() {
            return None;
        };

        // Extract the inner value first
        let mut value = if let Some(nested_ident) = &self.nested_ident {
            if field_args.default.is_some() {
                panic!("Cannot use `default` with `nested`.");
            }

            quote! {
                <#nested_ident as schematic::PartialConfig>::default_values(content)?
            }
        } else if let Some(expr) = &field_args.default {
            match expr {
                Expr::Array(_) | Expr::Call(_) | Expr::Macro(_) | Expr::Tuple(_) => {
                    quote! { #expr }
                }
                Expr::Path(func) => {
                    quote! { schematic::internal::handle_default_result(#func(context))? }
                }
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Str(string) => quote! {
                        schematic::internal::handle_default_result(std::convert::TryFrom::try_from(#string))?
                    },
                    other => quote! { #other },
                },
                invalid => {
                    panic!(
                        "Unsupported default value ({invalid:?}). May only provide literals, primitives, arrays, or tuples."
                    );
                }
            }
        } else {
            quote! {
                Default::default()
            }
        };

        // Then wrap with each layer
        for layer in self.layers.iter().rev() {
            value = match layer {
                Layer::Arc => quote! { Arc::new(#value) },
                Layer::Box => quote! { Box::new(#value) },
                Layer::Option => quote! { Some(#value) },
                Layer::Rc => quote! { Rc::new(#value) },
                Layer::Map(name) | Layer::Set(name) | Layer::Vec(name) | Layer::Unknown(name) => {
                    let collection = format_ident!("{name}");

                    quote! { #collection::default() }
                }
            };
        }

        Some(quote! {
            Some(#value)
        })
    }
}

fn extract_type_information(
    ty: &Type,
    layers: &mut Vec<Layer>,
    nested_ident: bool,
) -> Option<Ident> {
    // We don't need to traverse other types, just paths
    let Type::Path(ty_path) = ty else {
        return None;
    };

    // Extract the last segment of the path, for example `Option`,
    // instead of the full path `std::option::Option`
    let last_segment = ty_path.path.segments.last().unwrap();

    match &last_segment.arguments {
        // We've reached the final segment
        PathArguments::None => {
            if nested_ident {
                return Some(last_segment.ident.clone());
            }
        }

        // Attempt to drill deeper down
        PathArguments::AngleBracketed(args) => {
            extract_layer(last_segment, layers);

            if let Some(GenericArgument::Type(inner_ty)) = args.args.last() {
                return extract_type_information(inner_ty, layers, nested_ident);
            }
        }

        // What to do here, anything?
        PathArguments::Parenthesized(_) => {}
    };

    None
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
