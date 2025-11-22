use crate::args::NestedArg;
use crate::utils::{ImplResult, to_type_string};
use quote::{ToTokens, format_ident, quote};
use syn::{GenericArgument, Ident, PathArguments, PathSegment, Type};

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
pub struct Value {
    pub inner_ty: Option<Type>,
    pub layers: Vec<Layer>,
    pub nested: bool,
    pub nested_ident: Option<Ident>,
    pub ty: Type,
    pub ty_string: String,
}

impl Value {
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

        let mut value = Value {
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

    pub fn impl_partial_finalize_nested(&self, layer_var: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let mut value = quote! { #layer_var.finalize(context)? };

        // Then wrap with each layer
        if !self.layers.is_empty() {
            for layer in self.layers.iter().rev() {
                value = match layer {
                    Layer::Arc => quote! { Arc::new(#value) },
                    Layer::Rc => quote! { Rc::new(#value) },
                    Layer::Box => quote! { Box::new(#value) },
                    Layer::Option => quote! {
                       match #layer_var {
                           Some(#layer_var) => #value,
                           None => None
                       }
                    },
                    Layer::Map(name) => {
                        let collection = format_ident!("{name}");

                        quote! {
                            {
                                let mut map = #collection::default();
                                for (key, value) in #layer_var {
                                    map.insert(key, value.finalize(context)?);
                                }
                                map
                            }
                        }
                    }
                    Layer::Set(name) => {
                        let collection = format_ident!("{name}");

                        quote! {
                            {
                                let mut set = #collection::default();
                                for item in #layer_var {
                                    set.insert(item.finalize(context)?);
                                }
                                set
                            }
                        }
                    }
                    Layer::Vec(name) => {
                        let collection = format_ident!("{name}");

                        quote! {
                            {
                                let mut list = #collection::default();
                                for item in #layer_var {
                                    list.push(item.finalize(context)?);
                                }
                                list
                            }
                        }
                    }
                    Layer::Unknown(name) => {
                        let collection = format_ident!("{name}");

                        quote! { #collection::default() }
                    }
                };
            }
        }

        res.value = value;
        res
    }

    #[cfg(not(feature = "validate"))]
    pub fn impl_partial_validate_nested(
        &self,
        _path_key: &str,
        _setting_var: &Ident,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    #[cfg(feature = "validate")]
    pub fn impl_partial_validate_nested(&self, path_key: &str, setting_var: &Ident) -> ImplResult {
        if self.layers.len() >= 2
            && self
                .layers
                .get(1)
                .is_some_and(|layer| matches!(layer, Layer::Option))
        {
            return ImplResult::skipped();
        }

        let mut value = quote! {
            validate.nested(#path_key, #setting_var);
        };

        for layer in self.layers.iter().rev() {
            match layer {
                Layer::Arc | Layer::Box | Layer::Option | Layer::Rc => {
                    // Nothing?
                }
                Layer::Map(_) => {
                    value = quote! {
                        validate.nested_map(#path_key, #setting_var.iter());
                    };
                }
                Layer::Set(_) | Layer::Vec(_) => {
                    value = quote! {
                        validate.nested_list(#path_key, #setting_var.iter());
                    };
                }
                Layer::Unknown(_) => {
                    return ImplResult::skipped();
                }
            };
        }

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
