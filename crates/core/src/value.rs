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

impl Layer {
    /// Pointer types that only wrap a value, and are stripped from the
    /// partial and re-applied when constructing the final configuration.
    pub fn is_wrapper(&self) -> bool {
        matches!(self, Self::Arc | Self::Box | Self::Rc)
    }

    pub fn is_collection(&self) -> bool {
        matches!(
            self,
            Self::Map(_) | Self::Set(_) | Self::Vec(_) | Self::Unknown(_)
        )
    }
}

#[derive(Debug)]
pub struct Value {
    pub inner_ty: Option<Type>,
    pub layers: Vec<Layer>,
    pub nested: bool,
    pub nested_ident: Option<Ident>,
    /// Whether wrapper layers are stripped from the partial. Disabled when a
    /// wrapper contains an unsized type, like `Box<str>`, as the inner value
    /// cannot exist on its own.
    pub strip_wrappers: bool,
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
            strip_wrappers: !has_unsized_wrapper(&ty),
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

        if !self.nested {
            return;
        }

        let Some(nested_ident) = &self.nested_ident else {
            panic!(
                "Unable to extract the nested configuration identifier from `{}`. Try explicitly passing the identifier with `nested = ConfigName`.",
                self.ty_string
            )
        };

        // Primitives can never implement `Config`, so catch them here instead
        // of failing later with an obscure trait error
        if is_primitive_ident(nested_ident) {
            panic!(
                "Nested configurations must be a `Config` type, received `{}`.",
                self.ty_string
            )
        }
    }

    pub fn get_inner_type(&self) -> &Type {
        self.inner_ty.as_ref().unwrap_or(&self.ty)
    }

    /// Return the type to use within the partial, which replaces the nested
    /// configuration type with its partial counterpart, and strips wrapper
    /// types. For example, `Arc<Vec<Config>>` becomes
    /// `Vec<<Config as schematic::Config>::Partial>`.
    pub fn get_partial_type(&self) -> Type {
        let mut ty = self.ty.clone();

        if let Some(nested_ident) = &self.nested_ident {
            ty = replace_nested_type(&ty, nested_ident);
        }

        if self.strip_wrappers {
            ty = strip_wrapper_types(&ty);
        }

        ty
    }

    /// Return the layers that exist within the partial, which excludes
    /// wrappers, as those are only applied to the final configuration.
    pub fn get_partial_layers(&self) -> Vec<&Layer> {
        self.layers
            .iter()
            .filter(|layer| !self.strip_wrappers || !layer.is_wrapper())
            .collect()
    }

    pub fn is_collection(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_collection())
    }

    /// Whether a value can be sourced from the environment. Only a bare
    /// type, or one wrapped in a single `Option`, can be parsed from a
    /// string, unless a `parse_env` function handles the conversion itself.
    pub fn supports_env_value(&self, has_parse_env: bool) -> bool {
        let layers = self.get_partial_layers();

        has_parse_env || layers.is_empty() || (layers.len() == 1 && self.is_outer_option_wrapped())
    }

    /// Whether the partial's outermost layer is an `Option`, in which case
    /// it doubles as the partial's own optionality.
    pub fn is_outer_option_wrapped(&self) -> bool {
        self.get_partial_layers()
            .first()
            .is_some_and(|layer| **layer == Layer::Option)
    }

    /// Whether converting from a partial requires rebuilding the value,
    /// either to convert nested partials or to re-apply stripped wrappers.
    pub fn requires_from_partial_mapping(&self) -> bool {
        self.nested_ident.is_some()
            || (self.strip_wrappers && self.layers.iter().any(|layer| layer.is_wrapper()))
    }

    /// Generate the value for the final configuration, by converting nested
    /// partials and re-applying any wrappers that the partial stripped.
    pub fn impl_full_from_partial_value(&self, data_var: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let mut value = match &self.nested_ident {
            Some(config) => quote! { #config::from_partial(#data_var) },
            None => quote! { #data_var },
        };

        // Then wrap with each layer, from the innermost to the outermost.
        // Wrappers are constructed here, as they don't exist in the partial.
        for layer in self.layers.iter().rev() {
            if layer.is_wrapper() && !self.strip_wrappers {
                continue;
            }

            value = match layer {
                Layer::Arc => quote! { Arc::new(#value) },
                Layer::Rc => quote! { Rc::new(#value) },
                Layer::Box => quote! { Box::new(#value) },
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

        // Wrappers don't exist in the partial, and the first `Option` layer is
        // represented by the partial's own `Option`, so skip it when the
        // caller has already unwrapped it
        let mut layers = self.get_partial_layers();

        if skip_outer_option && self.is_outer_option_wrapped() {
            layers.remove(0);
        }

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

    /// Generate a merge for a nested configuration.
    ///
    /// When `optional` is true, `prev` and `next` are already wrapped in an
    /// `Option` by the partial, so a `MergeManager` call is generated
    /// instead of a statement.
    pub fn impl_partial_merge_nested(
        &self,
        prev: &TokenStream,
        next: &TokenStream,
        optional: bool,
    ) -> ImplResult {
        let mut res = ImplResult::default();
        let outer_option = self.is_outer_option_wrapped();

        // The outermost `Option` is handled by the manager. Wrappers don't
        // exist in the partial, so anything else is unmergeable.
        let mut layers = self.get_partial_layers();

        if outer_option {
            layers.remove(0);
        }

        // Collections are replaced by the caller, so only `Option`s remain
        if !layers.is_empty() {
            panic!(
                "Nested configs may only be wrapped in an outermost `Option` when using `merge`."
            );
        }

        let manager = optional || outer_option;
        res.requires_internal = manager;
        res.value = if manager {
            quote! { .nested(#prev, #next)? }
        } else {
            quote! { #prev.merge(context, #next)?; }
        };

        res
    }

    #[cfg(not(feature = "validate"))]
    pub fn impl_partial_validate_nested(
        &self,
        _target: &TokenStream,
        _setting_var: &Ident,
        _variant: bool,
        _optional: bool,
    ) -> ImplResult {
        ImplResult::skipped()
    }

    /// Generate a validation for a nested configuration, unwrapping each
    /// layer of the setting until the inner partials can be validated.
    ///
    /// The `target` identifies the setting within error paths, and is either
    /// a key, or a variant name and position when `variant` is true.
    ///
    /// When `optional` is true, the setting has already been unwrapped
    /// from the partial's `Option`, so the outermost `Option` layer is skipped.
    #[cfg(feature = "validate")]
    pub fn impl_partial_validate_nested(
        &self,
        target: &TokenStream,
        setting_var: &Ident,
        variant: bool,
        optional: bool,
    ) -> ImplResult {
        let prefix = if variant { "nested_variant" } else { "nested" };
        let nested = format_ident!("{prefix}");
        let nested_list = format_ident!("{prefix}_list");
        let nested_map = format_ident!("{prefix}_map");

        // Wrappers don't exist in the partial, so only the structural
        // layers need to be traversed
        let mut layers = self.get_partial_layers();
        let outer_option = self.is_outer_option_wrapped();

        if outer_option {
            layers.remove(0);
        }

        // Nested `Option`s cannot be validated, as the inner value may not exist
        if layers.first().is_some_and(|layer| **layer == Layer::Option) {
            return ImplResult::skipped();
        }

        // Then unwrap each layer, from the outermost to the innermost,
        // stopping at the first collection
        let mut setting = quote! { #setting_var };
        let mut value = None;

        for (index, layer) in layers.iter().enumerate() {
            match layer {
                Layer::Arc | Layer::Box | Layer::Rc => {
                    setting = quote! { #setting.as_ref() };
                }
                Layer::Map(_) | Layer::Set(_) | Layer::Vec(_) => {
                    // An item may itself be optional, as in `Vec<Option<T>>`,
                    // and a `None` has nothing to validate. Items are handed
                    // over as `Option`s either way, so that a missing one
                    // still holds its position in the reported path.
                    let optional_items = layers
                        .get(index + 1)
                        .is_some_and(|layer| **layer == Layer::Option);

                    value = Some(if matches!(layer, Layer::Map(_)) {
                        let items = if optional_items {
                            quote! { #setting.iter().map(|(key, value)| (key, value.as_ref())) }
                        } else {
                            quote! { #setting.iter().map(|(key, value)| (key, Some(value))) }
                        };

                        quote! { validate.#nested_map(#target, #items); }
                    } else {
                        let items = if optional_items {
                            quote! { #setting.iter().map(|item| item.as_ref()) }
                        } else {
                            quote! { #setting.iter().map(Some) }
                        };

                        quote! { validate.#nested_list(#target, #items); }
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
                validate.#nested(#target, #setting);
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

fn is_wrapper_ident(ident: &Ident) -> bool {
    ident == "Arc" || ident == "Box" || ident == "Rc"
}

fn is_primitive_ident(ident: &Ident) -> bool {
    [
        "String", "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str",
        "u8", "u16", "u32", "u64", "u128", "usize",
    ]
    .iter()
    .any(|primitive| ident == primitive)
}

/// Whether a type cannot exist without being wrapped in a pointer,
/// like `str`, `[T]`, and `dyn Trait`.
fn is_unsized_type(ty: &Type) -> bool {
    match ty {
        Type::Slice(_) | Type::TraitObject(_) => true,
        Type::Path(ty_path) => ty_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "str"),
        _ => false,
    }
}

/// Whether any wrapper within the type contains an unsized value,
/// in which case the wrappers cannot be stripped from the partial.
fn has_unsized_wrapper(ty: &Type) -> bool {
    let Type::Path(ty_path) = ty else {
        return false;
    };

    let Some(last_segment) = ty_path.path.segments.last() else {
        return false;
    };

    let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return false;
    };

    args.args.iter().any(|arg| {
        let GenericArgument::Type(inner_ty) = arg else {
            return false;
        };

        (is_wrapper_ident(&last_segment.ident) && is_unsized_type(inner_ty))
            || has_unsized_wrapper(inner_ty)
    })
}

/// Remove all wrapper types, so that `Arc<Vec<Box<T>>>` becomes `Vec<T>`.
fn strip_wrapper_types(ty: &Type) -> Type {
    let Type::Path(ty_path) = ty else {
        return ty.clone();
    };

    let Some(last_segment) = ty_path.path.segments.last() else {
        return ty.clone();
    };

    let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return ty.clone();
    };

    // Replace the wrapper with the type it wraps
    if is_wrapper_ident(&last_segment.ident)
        && let Some(GenericArgument::Type(inner_ty)) = args.args.last()
    {
        return strip_wrapper_types(inner_ty);
    }

    // Otherwise drill into each argument, like the value of a map
    let mut ty_path = ty_path.clone();
    let last_segment = ty_path.path.segments.last_mut().unwrap();

    if let PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
        for arg in args.args.iter_mut() {
            if let GenericArgument::Type(inner_ty) = arg {
                *inner_ty = strip_wrapper_types(inner_ty);
            }
        }
    }

    Type::Path(ty_path)
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
