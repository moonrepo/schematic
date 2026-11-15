use crate::args::NestedArg;
use crate::utils::{ImplResult, to_type_string};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprPath, GenericArgument, Ident, Lit, Path, PathArguments, PathSegment, Type,
    parse_quote,
};

/// Whether a path names a value -- an enum variant, unit struct, or constant --
/// rather than a function to hand the context to.
///
/// Both arrive as a bare `Expr::Path`, and nothing at derive time can resolve
/// which one it is, so this leans on Rust's naming conventions: a function is
/// `snake_case`, while everything that names a value is `PascalCase` or
/// `SCREAMING_SNAKE_CASE`. A handler that ignored that would already be earning
/// a `non_snake_case` warning of its own.
pub fn path_names_a_value(path: &Path) -> bool {
    path.segments
        .last()
        .and_then(|segment| segment.ident.to_string().chars().next())
        .is_some_and(|char| char.is_uppercase())
}

pub fn wrap_layer(layer: &Layer, value: TokenStream) -> TokenStream {
    match layer {
        Layer::Arc => quote! { std::sync::Arc::new(#value) },
        Layer::Box => quote! { std::boxed::Box::new(#value) },
        Layer::Option => quote! { Some(#value) },
        Layer::Rc => quote! { std::rc::Rc::new(#value) },
        // Collections reset to empty, discarding the inner value. The type is
        // inferred, so that it never has to be in scope by name.
        Layer::Map(_) | Layer::Set(_) | Layer::Vec(_) | Layer::Unknown(_) => {
            quote! { Default::default() }
        }
    }
}

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
    /// Whether the `Box` around a nested config that holds the container
    /// itself, like `Option<Box<Self>>`, is kept in the partial. Stripping it
    /// would give the partial an infinite size.
    pub keep_box: bool,
    /// A function from `#[setting(nested, context)]` that maps the parent's
    /// context to the nested config's own, which may be a different type.
    pub context_fn: Option<ExprPath>,
    /// Whether wrapper layers are stripped from the partial. Disabled when a
    /// wrapper contains an unsized type, like `Box<str>`, as the inner value
    /// cannot exist on its own.
    pub strip_wrappers: bool,
    pub ty: Type,
    pub ty_string: String,
}

impl Value {
    /// Analyze a setting's type. The `self_ty` is the container's own type,
    /// which replaces any `Self`, as within the partial `Self` would mean
    /// the partial instead.
    pub fn new(mut ty: Type, nested_arg: Option<&NestedArg>, self_ty: &Type) -> Self {
        replace_self_type(&mut ty, self_ty);

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
            context_fn: None,
            inner_ty: None,
            keep_box: false,
            nested,
            nested_ident,
            layers: vec![],
            strip_wrappers: !has_unsized_wrapper(&ty),
            ty_string,
            ty,
        };
        value.extract_type_information();
        value.detect_recursion(self_ty);
        value
    }

    /// Set the function that maps the parent's context to the nested config's.
    pub fn with_context(mut self, context_fn: Option<ExprPath>) -> Self {
        if context_fn.is_some() && !self.nested {
            panic!("Cannot use `context` without `nested`.");
        }

        self.context_fn = context_fn;
        self
    }

    /// The context a nested config receives, which is the parent's own,
    /// unless `#[setting(nested, context)]` maps it to another.
    pub fn get_nested_context(&self) -> TokenStream {
        match &self.context_fn {
            Some(func) => quote! { #func(context) },
            None => quote! { context },
        }
    }

    /// A manager method that passes a nested config its context, with the
    /// extra argument it takes, as only a mapped context is passed explicitly.
    fn get_nested_method(&self, name: &str) -> (Ident, TokenStream) {
        match &self.context_fn {
            Some(func) => (
                quote::format_ident!("{name}_with"),
                quote! { , #func(context) },
            ),
            None => (quote::format_ident!("{name}"), quote! {}),
        }
    }

    /// A nested config holding the container itself needs a level of
    /// indirection, which a collection provides, and otherwise a `Box`
    /// that is kept in the partial.
    fn detect_recursion(&mut self, self_ty: &Type) {
        let Some(nested_ident) = &self.nested_ident else {
            return;
        };

        let Type::Path(self_path) = self_ty else {
            return;
        };

        if self_path
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != *nested_ident)
            || self.is_collection()
        {
            return;
        }

        if !self.layers.contains(&Layer::Box) {
            panic!(
                "A nested setting that holds its own type must be wrapped in a `Box` or a collection, like `Option<Box<Self>>` or `Vec<Self>`, received `{}`.",
                self.ty_string
            );
        }

        self.keep_box = true;
    }

    /// The type of the nested config, qualified as it was written, so that it
    /// resolves without its partial being in scope.
    pub fn get_nested_type(&self) -> Option<TokenStream> {
        let nested_ident = self.nested_ident.as_ref()?;

        if let Some(Type::Path(inner)) = &self.inner_ty
            && inner
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == *nested_ident)
        {
            return Some(quote! { #inner });
        }

        Some(quote! { #nested_ident })
    }

    /// The partial type of the nested config, as an associated type of the
    /// config itself, so that it resolves without being in scope by name.
    pub fn get_nested_partial_type(&self) -> Option<TokenStream> {
        let ty = self.get_nested_type()?;

        Some(quote! { <#ty as schematic::Config>::Partial })
    }

    pub fn extract_type_information(&mut self) {
        // A nested config may be generic, like `Wrapper<String>`, so the walk
        // stops at the first type that isn't a layer, and that's the config
        let nested = self.nested;

        extract_type_information(&self.ty, &mut self.layers, nested, |ty, segment| {
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

        if nested_ident == "MergeList" {
            panic!(
                "A `MergeList` can't hold nested configs, received `{}`.",
                self.ty_string
            )
        }

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
            ty = strip_wrapper_types(&ty, self.keep_box);
        }

        ty
    }

    /// Return the layers that exist within the partial, which excludes
    /// wrappers, as those are only applied to the final configuration.
    pub fn get_partial_layers(&self) -> Vec<&Layer> {
        self.layers
            .iter()
            .filter(|layer| {
                !self.strip_wrappers
                    || !layer.is_wrapper()
                    || (self.keep_box && **layer == Layer::Box)
            })
            .collect()
    }

    pub fn is_collection(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_collection())
    }

    /// Whether the setting is a `MergeList`, known by name like the layers
    /// are, so that it's merged with `merge::merge_list` without a `merge` of
    /// its own. Only the outermost type counts, within an `Option`, as a
    /// collection of them is replaced like any other. An alias isn't known.
    pub fn is_merge_list(&self) -> bool {
        if self.nested {
            return false;
        }

        let mut layers = self.get_partial_layers();

        if self.is_outer_option_wrapped() {
            layers.remove(0);
        }

        matches!(layers.first(), Some(Layer::Unknown(name)) if name == "MergeList")
    }

    /// The partial type at each of its layers, from the outermost inward,
    /// like `Option<Vec<T>>` and then `Vec<T>`.
    fn get_partial_layer_types(&self) -> Vec<Type> {
        let mut ty = self.get_partial_type();
        let mut types = vec![];

        for _ in self.get_partial_layers() {
            let inner = get_last_type_argument(&ty);

            types.push(ty);
            ty = inner.unwrap_or_else(|| types.last().unwrap().to_owned());
        }

        types
    }

    /// Generate an explicit default value from `#[setting(default = value)]`,
    /// wrapped with the provided layers. A value is written up to the
    /// outermost collection, so only the layers outside of it are applied.
    /// Handler functions are left to the caller, as what they return
    /// differs between a setting and a variant.
    pub fn impl_default_value(&self, expr: &Expr, layers: &[&Layer]) -> ImplResult {
        let mut res = ImplResult::default();

        let mut value = match expr {
            Expr::Array(_)
            | Expr::Call(_)
            | Expr::Macro(_)
            | Expr::Path(_)
            | Expr::Struct(_)
            | Expr::Tuple(_) => {
                quote! { #expr }
            }
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(string) => {
                    let ty = self.get_default_target_type(layers);
                    res.requires_internal = true;

                    quote! {
                        (&&DefaultString::<#ty>::new()).parse_default(#string)?
                    }
                }
                other => quote! { #other },
            },
            invalid => {
                panic!(
                    "Unsupported default value ({invalid:?}). May only provide literals, primitives, arrays, tuples, structs, paths, or calls."
                );
            }
        };

        let outer = layers
            .iter()
            .position(|layer| layer.is_collection())
            .map(|index| &layers[..index])
            .unwrap_or(layers);

        for layer in outer.iter().rev() {
            // A wrapper only remains in the partial when it can't be
            // stripped, like `Box<str>`, and the value converts into it
            if layer.is_wrapper() {
                continue;
            }

            value = wrap_layer(layer, value);
        }

        res.value = value;
        res
    }

    /// The type that a string literal default converts into. It's written
    /// inside every `Option` that is wrapped around it, while an unsized
    /// wrapper, like `Box<str>`, converts as a whole.
    fn get_default_target_type(&self, layers: &[&Layer]) -> Type {
        let mut ty = self.get_partial_type();

        // The caller may have already removed the partial's outer `Option`
        for _ in 0..self.get_partial_layers().len().saturating_sub(layers.len()) {
            ty = get_last_type_argument(&ty).unwrap_or(ty);
        }

        for layer in layers {
            if **layer != Layer::Option {
                break;
            }

            ty = get_last_type_argument(&ty).unwrap_or(ty);
        }

        ty
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
        let value = match self.get_nested_type() {
            Some(config) => quote! { <#config as schematic::Config>::from_partial(#data_var) },
            None => quote! { #data_var },
        };

        // Every layer of the full type is rebuilt. A wrapper the partial
        // stripped is constructed, while one it kept is unwrapped first, and
        // one around an unsized type is used as-is.
        let steps = self
            .layers
            .iter()
            .filter(|layer| !layer.is_wrapper() || self.strip_wrappers)
            .map(|layer| LayerStep {
                layer,
                ty: None,
                unwrap: self.keep_box && *layer == Layer::Box,
            })
            .collect::<Vec<_>>();

        ImplResult {
            value: map_layers(data_var, &steps, value, None, &self.ty_string),
            ..Default::default()
        }
    }

    /// Generate the finalization of a nested config, within the partial.
    ///
    /// The config is finalized in full, applying its own default values and
    /// environment variables, when it's an item of a collection, or `boundary`
    /// is true, as nothing above it has applied them. Otherwise its parent's
    /// `finalize` has merged them, as part of its own, so it's finalized as
    /// nested. An optional config is left `None` by its parent's defaults, so
    /// its default values are merged under it first.
    pub fn impl_partial_finalize_nested(
        &self,
        layer_var: &Ident,
        skip_outer_option: bool,
        boundary: bool,
    ) -> ImplResult {
        let skips_option = skip_outer_option && self.is_outer_option_wrapped();

        // The innermost value, and the `Result` it unwraps, when a collection
        // can map over that directly rather than `Ok(...?)`
        let context = self.get_nested_context();
        let (value, result) = if boundary || self.is_collection() {
            (
                quote! { #layer_var.finalize(#context)? },
                Some(quote! { #layer_var.finalize(#context) }),
            )
        } else if skips_option && let Some(partial) = self.get_nested_partial_type() {
            let value = quote! {
                {
                    let mut base = <#partial as schematic::PartialConfig>::default_values(#context)?
                        .unwrap_or_default();
                    base.merge(#context, #layer_var)?;
                    base.finalize_nested(#context)?
                }
            };

            (value, None)
        } else {
            (quote! { #layer_var.finalize_nested(#context)? }, None)
        };

        // Wrappers stripped from the partial don't exist in it, and the first
        // `Option` layer is the partial's own, so it's skipped when the caller
        // has already unwrapped it. Every remaining layer is rebuilt, and as
        // finalizing can fail, collections collect into a `Result` of the type
        // at that layer, which is named, as a `transform` may be generic.
        let mut layers = self.get_partial_layers();
        let mut layer_types = self.get_partial_layer_types();

        if skips_option {
            layers.remove(0);
            layer_types.remove(0);
        }

        let steps = layers
            .into_iter()
            .zip(layer_types)
            .map(|(layer, ty)| LayerStep {
                layer,
                ty: Some(ty),
                unwrap: true,
            })
            .collect::<Vec<_>>();

        ImplResult {
            value: map_layers(layer_var, &steps, value, result, &self.ty_string),
            ..Default::default()
        }
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

        // A recursive config keeps its `Box`, which is merged through
        let boxed = self.keep_box && layers.len() == 1 && *layers[0] == Layer::Box;

        // Collections are replaced by the caller, so only `Option`s remain
        if !layers.is_empty() && !boxed {
            panic!(
                "Nested configs may only be wrapped in an outermost `Option` when using `merge`."
            );
        }

        let manager = optional || outer_option;
        let context = self.get_nested_context();
        let (nested, nested_context) = self.get_nested_method("nested");
        let (nested_boxed, _) = self.get_nested_method("nested_boxed");

        res.requires_internal = manager;
        res.value = match (manager, boxed) {
            (true, true) => quote! { .#nested_boxed(#prev, #next #nested_context)? },
            (true, false) => quote! { .#nested(#prev, #next #nested_context)? },
            (false, true) => quote! { #prev.merge(#context, *#next)?; },
            (false, false) => quote! { #prev.merge(#context, #next)?; },
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
        let (nested, nested_context) =
            self.get_nested_method(if variant { "nested_variant" } else { "nested" });

        // Wrappers don't exist in the partial, so only the structural
        // layers need to be traversed
        let mut layers = self.get_partial_layers();
        let outer_option = self.is_outer_option_wrapped();

        if outer_option {
            layers.remove(0);
        }

        // Nested `Option`s cannot be validated, as the inner value may not
        // exist, and an unknown collection cannot be iterated
        if layers.first().is_some_and(|layer| **layer == Layer::Option)
            || layers
                .iter()
                .any(|layer| matches!(layer, Layer::Unknown(_)))
        {
            return ImplResult::skipped();
        }

        let mut value = if layers.iter().any(|layer| layer.is_collection()) {
            let (nested_at, nested_context) = self.get_nested_method("nested_at");

            impl_partial_validate_nested_items(
                target,
                setting_var,
                variant,
                &layers,
                quote! { validate.#nested_at(path, item #nested_context); },
            )
        } else {
            // A single value is validated at the setting's own path
            let mut setting = quote! { #setting_var };

            for layer in &layers {
                match layer {
                    Layer::Arc | Layer::Box | Layer::Rc => {
                        setting = quote! { #setting.as_ref() };
                    }
                    _ => {
                        return ImplResult::skipped();
                    }
                };
            }

            quote! {
                validate.#nested(#target, #setting #nested_context);
            }
        };

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

/// Generate a validation of every item within a collection of nested
/// configurations. Each layer is unwrapped from the innermost outwards, as
/// finalizing and merging do, so that any number of collections can be
/// nested, and each collection extends the path an item is reported at, like
/// `groups.name[0]` for a map of lists. An item that is `None` is skipped,
/// but still holds its position.
#[cfg(feature = "validate")]
fn impl_partial_validate_nested_items(
    target: &TokenStream,
    setting_var: &Ident,
    variant: bool,
    layers: &[&Layer],
    validate_item: TokenStream,
) -> TokenStream {
    let mut value = validate_item;

    for layer in layers.iter().rev() {
        value = match layer {
            Layer::Arc | Layer::Box | Layer::Rc => quote! {
                let item = item.as_ref();
                #value
            },
            Layer::Option => quote! {
                if let Some(item) = item {
                    #value
                }
            },
            Layer::Map(_) => quote! {
                for (key, item) in item {
                    let path = path.join_key(key);
                    #value
                }
            },
            Layer::Set(_) | Layer::Vec(_) => quote! {
                for (index, item) in item.iter().enumerate() {
                    let path = path.join_index(index);
                    #value
                }
            },
            Layer::Unknown(_) => unreachable!("Skipped by the caller."),
        };
    }

    let path = if variant {
        quote! { validate.variant_path(#target) }
    } else {
        quote! { validate.key_path(#target) }
    };

    quote! {
        {
            let path = #path;
            let item = #setting_var;
            #value
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
/// like `str`, `Path`, `OsStr`, `CStr`, `[T]`, and `dyn Trait`.
fn is_unsized_type(ty: &Type) -> bool {
    match ty {
        Type::Slice(_) | Type::TraitObject(_) => true,
        Type::Path(ty_path) => ty_path.path.segments.last().is_some_and(|segment| {
            segment.ident == "str"
                || segment.ident == "Path"
                || segment.ident == "OsStr"
                || segment.ident == "CStr"
        }),
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
/// A `Box` is kept when `keep_box` is true, for a recursive config.
fn strip_wrapper_types(ty: &Type, keep_box: bool) -> Type {
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
        && !(keep_box && last_segment.ident == "Box")
        && let Some(GenericArgument::Type(inner_ty)) = args.args.last()
    {
        return strip_wrapper_types(inner_ty, keep_box);
    }

    // Otherwise drill into each argument, like the value of a map
    let mut ty_path = ty_path.clone();
    let last_segment = ty_path.path.segments.last_mut().unwrap();

    if let PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
        for arg in args.args.iter_mut() {
            if let GenericArgument::Type(inner_ty) = arg {
                *inner_ty = strip_wrapper_types(inner_ty, keep_box);
            }
        }
    }

    Type::Path(ty_path)
}

/// Replace the nested config within a type with its partial, walking the same
/// layers that `extract_type_information` does, so that a generic config like
/// `Wrapper<String>` is replaced as a whole.
fn replace_nested_type(ty: &Type, nested_ident: &Ident) -> Type {
    // We don't need to traverse other types, just paths
    let Type::Path(ty_path) = ty else {
        return ty.clone();
    };

    let last_segment = ty_path.path.segments.last().unwrap();

    // A layer holds the config, so drill into it
    if let PathArguments::AngleBracketed(_) = &last_segment.arguments
        && is_layer_segment(last_segment)
    {
        let mut ty_path = ty_path.clone();
        let last_segment = ty_path.path.segments.last_mut().unwrap();

        if let PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
            for arg in args.args.iter_mut() {
                if let GenericArgument::Type(inner_ty) = arg {
                    *inner_ty = replace_nested_type(inner_ty, nested_ident);
                }
            }
        }

        return Type::Path(ty_path);
    }

    // Otherwise this is the config itself
    if last_segment.ident == *nested_ident {
        return parse_quote! { <#ty_path as schematic::Config>::Partial };
    }

    ty.clone()
}

fn extract_type_information(
    ty: &Type,
    layers: &mut Vec<Layer>,
    stop_at_unknown: bool,
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

        // A generic that isn't a layer is the type itself, when asked
        PathArguments::AngleBracketed(_) if stop_at_unknown && !is_layer_segment(last_segment) => {
            on_last(ty, last_segment);
        }

        // Attempt to drill deeper down
        PathArguments::AngleBracketed(args) => {
            extract_layer(last_segment, layers);

            if let Some(GenericArgument::Type(inner_ty)) = args.args.last() {
                extract_type_information(inner_ty, layers, stop_at_unknown, on_last);
            }
        }

        // What to do here, anything?
        PathArguments::Parenthesized(_) => {}
    };
}

/// Whether a generic type is a layer the derive walks through, a wrapper,
/// an `Option`, or a known collection, rather than a type of its own.
fn is_layer_segment(segment: &PathSegment) -> bool {
    let mut layers = vec![];

    extract_layer(segment, &mut layers);

    !matches!(layers.first(), Some(Layer::Unknown(_)))
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

        if ident.ends_with("Vec")
            || ident == "VecDeque"
            || ident == "LinkedList"
            || ident == "BinaryHeap"
        {
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

/// A layer that `map_layers` rebuilds a value through.
struct LayerStep<'a> {
    layer: &'a Layer,
    /// The type at this layer, which a collection collects into when mapping
    /// its items can fail. Without one, mapping is infallible.
    ty: Option<Type>,
    /// Whether the value already holds this wrapper, so it's unwrapped
    /// before being rebuilt, rather than only constructed.
    unwrap: bool,
}

/// Map a value through each layer, from the innermost outward, rebuilding each
/// layer around the value within it. Collections are rebuilt with `collect`,
/// so any `FromIterator` collection works without being named.
///
/// When the steps carry a type, mapping is fallible: the innermost `value`
/// uses `?`, and `result`, when given, is the `Result` it unwraps, which a
/// collection maps over directly rather than `Ok(...?)`.
fn map_layers(
    var: &Ident,
    steps: &[LayerStep],
    mut value: TokenStream,
    mut result: Option<TokenStream>,
    ty_string: &str,
) -> TokenStream {
    for step in steps.iter().rev() {
        let fallible_ty = step.ty.as_ref();

        value = match step.layer {
            Layer::Arc | Layer::Box | Layer::Rc => {
                let (wrapper, unwrapped) = match step.layer {
                    Layer::Arc => (
                        quote! { std::sync::Arc },
                        quote! { std::sync::Arc::unwrap_or_clone(#var) },
                    ),
                    Layer::Rc => (
                        quote! { std::rc::Rc },
                        quote! { std::rc::Rc::unwrap_or_clone(#var) },
                    ),
                    _ => (quote! { std::boxed::Box }, quote! { *#var }),
                };

                if step.unwrap {
                    quote! {
                        {
                            let #var = #unwrapped;
                            #wrapper::new(#value)
                        }
                    }
                } else {
                    quote! { #wrapper::new(#value) }
                }
            }
            Layer::Option => quote! {
                match #var {
                    Some(#var) => Some(#value),
                    None => None
                }
            },
            Layer::Map(_) => match fallible_ty {
                Some(ty) => {
                    let collected = quote! {
                        #var
                            .into_iter()
                            .map(|(key, #var)| Ok((key, #value)))
                            .collect::<std::result::Result<#ty, schematic::ConfigError>>()
                    };

                    result = Some(collected.clone());

                    quote! { #collected? }
                }
                None => quote! {
                    #var
                        .into_iter()
                        .map(|(key, #var)| (key, #value))
                        .collect()
                },
            },
            Layer::Set(_) | Layer::Vec(_) => match fallible_ty {
                Some(ty) => {
                    let item = result.take().unwrap_or_else(|| quote! { Ok(#value) });
                    let collected = quote! {
                        #var
                            .into_iter()
                            .map(|#var| #item)
                            .collect::<std::result::Result<#ty, schematic::ConfigError>>()
                    };

                    result = Some(collected.clone());

                    quote! { #collected? }
                }
                None => quote! {
                    #var
                        .into_iter()
                        .map(|#var| #value)
                        .collect()
                },
            },
            Layer::Unknown(name) => panic_unknown_collection(name, ty_string),
        };

        // Only a collection's own result can be mapped over by the next
        if !step.layer.is_collection() {
            result = None;
        }
    }

    value
}

/// Panic for a type that wraps a nested config, or a wrapper the partial
/// strips, but can't be rebuilt, as it isn't a known collection.
fn panic_unknown_collection(name: &str, ty_string: &str) -> TokenStream {
    panic!(
        "Unable to convert `{ty_string}`, as `{name}` is not a known collection. A collection must be named like `*Vec`, `*Set`, or `*Map`, or be a `VecDeque`, `LinkedList`, or `BinaryHeap`."
    );
}

/// The last type argument of a path type, like `T` for `Option<T>`.
fn get_last_type_argument(ty: &Type) -> Option<Type> {
    let Type::Path(ty_path) = ty else {
        return None;
    };

    let PathArguments::AngleBracketed(args) = &ty_path.path.segments.last()?.arguments else {
        return None;
    };

    args.args.iter().rev().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner.to_owned()),
        _ => None,
    })
}

/// Replace every `Self` within a type with the container's own type.
pub fn replace_self_type(ty: &mut Type, self_ty: &Type) {
    match ty {
        Type::Path(ty_path) => {
            if ty_path.qself.is_none() && ty_path.path.is_ident("Self") {
                *ty = self_ty.to_owned();

                return;
            }

            if let Some(qself) = &mut ty_path.qself {
                replace_self_type(&mut qself.ty, self_ty);
            }

            for segment in ty_path.path.segments.iter_mut() {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in args.args.iter_mut() {
                        if let GenericArgument::Type(inner) = arg {
                            replace_self_type(inner, self_ty);
                        }
                    }
                }
            }
        }
        Type::Array(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Group(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Paren(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Ptr(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Reference(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Slice(inner) => replace_self_type(&mut inner.elem, self_ty),
        Type::Tuple(inner) => {
            for elem in inner.elems.iter_mut() {
                replace_self_type(elem, self_ty);
            }
        }
        _ => {}
    }
}
