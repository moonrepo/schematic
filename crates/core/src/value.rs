use crate::args::NestedArg;
use crate::utils::{ImplResult, to_type_string};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{GenericArgument, Ident, PathArguments, PathSegment, Type, parse_quote};

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

    /// Return the type to use within the partial, which replaces the nested
    /// configuration type with its partial counterpart, while preserving all
    /// wrapping layers. For example, `Vec<Config>` becomes
    /// `Vec<<Config as schematic::Config>::Partial>`.
    pub fn get_partial_type(&self) -> Type {
        match &self.nested_ident {
            Some(nested_ident) => replace_nested_type(&self.ty, nested_ident),
            None => self.ty.clone(),
        }
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

    pub fn impl_full_from_partial_nested(&self, data_var: &Ident) -> ImplResult {
        let Some(config) = &self.nested_ident else {
            return ImplResult::skipped();
        };

        let mut res = ImplResult::default();
        let mut value = quote! { #config::from_partial(#data_var) };

        // Then wrap with each layer
        if !self.layers.is_empty() {
            for layer in self.layers.iter().rev() {
                value = match layer {
                    Layer::Arc => quote! {
                        {
                            let #data_var = Arc::unwrap_or_clone(#data_var);
                            Arc::new(#value)
                        }
                    },
                    Layer::Rc => quote! {
                        {
                            let #data_var = Rc::unwrap_or_clone(#data_var);
                            Rc::new(#value)
                        }
                    },
                    Layer::Box => quote! {
                        {
                            let #data_var = *#data_var;
                            Box::new(#value)
                        }
                    },
                    Layer::Option => quote! {
                        match #data_var {
                            Some(#data_var) => Some(#value),
                            None => None
                        }
                    },
                    Layer::Map(name) => {
                        let collection = format_ident!("{name}");

                        quote! {
                            {
                                let mut map = #collection::default();
                                for (key, #data_var) in #data_var {
                                    map.insert(key, #value);
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
                                for #data_var in #data_var {
                                    set.insert(#value);
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
                                for #data_var in #data_var {
                                    list.push(#value);
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

    pub fn impl_partial_finalize_nested(
        &self,
        layer_var: &Ident,
        skip_outer_option: bool,
    ) -> ImplResult {
        let mut res = ImplResult::default();
        let mut value = quote! { #layer_var.finalize(context)? };

        // The first `Option` layer is represented by the partial's own
        // `Option`, so skip it when the caller has already unwrapped it
        let layers = if skip_outer_option && self.is_outer_option_wrapped() {
            &self.layers[1..]
        } else {
            &self.layers[..]
        };

        // Then wrap with each layer
        if !layers.is_empty() {
            for layer in layers.iter().rev() {
                value = match layer {
                    Layer::Arc => quote! {
                        {
                            let #layer_var = Arc::unwrap_or_clone(#layer_var);
                            Arc::new(#value)
                        }
                    },
                    Layer::Rc => quote! {
                        {
                            let #layer_var = Rc::unwrap_or_clone(#layer_var);
                            Rc::new(#value)
                        }
                    },
                    Layer::Box => quote! {
                        {
                            let #layer_var = *#layer_var;
                            Box::new(#value)
                        }
                    },
                    Layer::Option => quote! {
                        match #layer_var {
                            Some(#layer_var) => Some(#value),
                            None => None
                        }
                    },
                    Layer::Map(name) => {
                        let collection = format_ident!("{name}");

                        quote! {
                            {
                                let mut map = #collection::default();
                                for (key, #layer_var) in #layer_var {
                                    map.insert(key, #value);
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
                                for #layer_var in #layer_var {
                                    set.insert(#value);
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
                                for #layer_var in #layer_var {
                                    list.push(#value);
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

    /// Generate a merge for a nested configuration, unwrapping each layer of
    /// `prev` and `next` until the inner partials can be merged.
    ///
    /// When `optional` is true, `prev` and `next` are already wrapped in an
    /// `Option` by the partial, so a `MergeManager` call is generated instead
    /// of a statement, and the outermost `Option` layer is skipped.
    pub fn impl_partial_merge_nested(
        &self,
        prev: &TokenStream,
        next: &TokenStream,
        optional: bool,
    ) -> ImplResult {
        let mut res = ImplResult::default();
        let outer_option = self.is_outer_option_wrapped();

        // The outermost `Option` is handled by the manager,
        // while the remaining layers must be unwrapped manually
        let manager = optional || outer_option;
        let wrappers = if outer_option {
            &self.layers[1..]
        } else {
            &self.layers[..]
        };

        // Nothing to unwrap, so merge directly
        if wrappers.is_empty() {
            res.requires_internal = manager;
            res.value = if manager {
                quote! { .nested(#prev, #next)? }
            } else {
                quote! { #prev.merge(context, #next)?; }
            };

            return res;
        }

        // Then unwrap each layer, from the outermost to the innermost
        let mut place = if manager {
            quote! { prev }
        } else {
            quote! { *#prev }
        };
        let mut value = if manager {
            quote! { next }
        } else {
            quote! { #next }
        };

        for layer in wrappers {
            match layer {
                Layer::Box => {
                    place = quote! { *#place };
                    value = quote! { *#value };
                }
                Layer::Arc => {
                    place = quote! { *Arc::make_mut(&mut #place) };
                    value = quote! { Arc::unwrap_or_clone(#value) };
                }
                Layer::Rc => {
                    place = quote! { *Rc::make_mut(&mut #place) };
                    value = quote! { Rc::unwrap_or_clone(#value) };
                }
                Layer::Option => {
                    panic!(
                        "Nested configs may only be wrapped in an outermost `Option` when using `merge`."
                    );
                }
                Layer::Map(_) | Layer::Set(_) | Layer::Vec(_) | Layer::Unknown(_) => {
                    panic!("Collections with nested configs must manually define `merge`.");
                }
            };
        }

        res.requires_internal = manager;
        res.value = if manager {
            quote! {
                .apply_with(#prev, #next, |mut prev, next, context| {
                    (#place).merge(context, #value)
                        .map_err(|error| schematic::MergeError(error.to_string()))?;

                    Ok(Some(prev))
                })?
            }
        } else {
            quote! {
                (#place).merge(context, #value)?;
            }
        };

        res
    }

    #[cfg(not(feature = "validate"))]
    pub fn impl_partial_validate_nested(
        &self,
        _path_key: &str,
        _setting_var: &Ident,
        _optional: bool,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    /// Generate a validation for a nested configuration, unwrapping each
    /// layer of the setting until the inner partials can be validated.
    ///
    /// When `optional` is true, the setting has already been unwrapped
    /// from the partial's `Option`, so the outermost `Option` layer is skipped.
    #[cfg(feature = "validate")]
    pub fn impl_partial_validate_nested(
        &self,
        path_key: &str,
        setting_var: &Ident,
        optional: bool,
    ) -> ImplResult {
        if self.layers.len() >= 2
            && self
                .layers
                .get(1)
                .is_some_and(|layer| matches!(layer, Layer::Option))
        {
            return ImplResult::skipped();
        }

        let outer_option = self.is_outer_option_wrapped();
        let layers = if outer_option {
            &self.layers[1..]
        } else {
            &self.layers[..]
        };

        // Then unwrap each layer, from the outermost to the innermost,
        // stopping at the first collection
        let mut setting = quote! { #setting_var };
        let mut value = None;

        for layer in layers {
            match layer {
                Layer::Arc | Layer::Box | Layer::Rc => {
                    setting = quote! { #setting.as_ref() };
                }
                Layer::Map(_) => {
                    value = Some(quote! {
                        validate.nested_map(#path_key, #setting.iter());
                    });
                    break;
                }
                Layer::Set(_) | Layer::Vec(_) => {
                    value = Some(quote! {
                        validate.nested_list(#path_key, #setting.iter());
                    });
                    break;
                }
                Layer::Option | Layer::Unknown(_) => {
                    return ImplResult::skipped();
                }
            };
        }

        let mut value = value.unwrap_or_else(|| {
            quote! {
                validate.nested(#path_key, #setting);
            }
        });

        // The outermost `Option` is a real value, so unwrap it
        if outer_option && !optional {
            value = quote! {
                if let Some(#setting_var) = #setting_var {
                    #value
                }
            };
        }

        ImplResult {
            value,
            ..Default::default()
        }
    }
}

fn replace_nested_type(ty: &Type, nested_ident: &Ident) -> Type {
    // We don't need to traverse other types, just paths
    let Type::Path(ty_path) = ty else {
        return ty.clone();
    };

    let last_segment = ty_path.path.segments.last().unwrap();

    match &last_segment.arguments {
        // We've reached the final segment, so replace it if it matches
        PathArguments::None => {
            if last_segment.ident == *nested_ident {
                parse_quote! { <#ty_path as schematic::Config>::Partial }
            } else {
                ty.clone()
            }
        }

        // Attempt to drill deeper down
        PathArguments::AngleBracketed(_) => {
            let mut ty_path = ty_path.clone();
            let last_segment = ty_path.path.segments.last_mut().unwrap();

            if let PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
                for arg in args.args.iter_mut() {
                    if let GenericArgument::Type(inner_ty) = arg {
                        *inner_ty = replace_nested_type(inner_ty, nested_ident);
                    }
                }
            }

            Type::Path(ty_path)
        }

        // What to do here, anything?
        PathArguments::Parenthesized(_) => ty.clone(),
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
