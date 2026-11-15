use crate::args::{
    PartialArg, SerdeContainerArgs, SerdeIoDirection, SerdeRenameArg, SerdeTagFormat,
};
use crate::field::Field;
use crate::utils::{ImplResult, is_inheritable_attribute, to_type_string, validate_case_format};
use crate::variant::Variant;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::ext::IdentExt;
use syn::{
    Attribute, Data, DeriveInput, ExprPath, Fields, Generics, Ident, Type, Visibility, parse_quote,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContainerMacro {
    Config,
    ConfigUnitEnum,
    Schematic,
}

// #[config()], #[schematic()]
#[derive(Debug, Default, FromDeriveInput)]
#[darling(default, attributes(config, schematic), supports(struct_any, enum_any))]
pub struct ContainerArgs {
    // config
    pub allow_unknown_fields: bool,
    pub before_parse: Option<String>,
    pub context: Option<ExprPath>,
    pub env_prefix: Option<String>,
    #[darling(multiple)]
    pub partial: Vec<PartialArg>,

    // serde
    pub rename: Option<SerdeRenameArg>,
    pub rename_all: Option<SerdeRenameArg>,
    pub rename_all_fields: Option<SerdeRenameArg>,
}

#[derive(Debug)]
pub struct Container {
    pub args: Rc<ContainerArgs>,
    pub inner: ContainerInner,
    pub serde_args: Rc<SerdeContainerArgs>,
    pub macro_type: ContainerMacro,

    // inherited
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
    pub ident: Ident,
    pub vis: Visibility,
}

impl Container {
    pub fn from(input: DeriveInput) -> Self {
        let args = Rc::new(ContainerArgs::from_derive_input(&input).unwrap());
        let serde_args = Rc::new(SerdeContainerArgs::from_derive_input(&input).unwrap());

        // Replaces `Self` within settings, as it means the partial within the partial
        let self_ty: Type = {
            let ident = &input.ident;
            let (_, ty_generics, _) = input.generics.split_for_impl();

            parse_quote! { #ident #ty_generics }
        };

        let inner = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => ContainerInner::NamedStruct {
                    fields: fields
                        .named
                        .into_iter()
                        .map(|data| Field::new(data, args.clone(), serde_args.clone(), &self_ty))
                        .collect(),
                },
                Fields::Unnamed(fields) => ContainerInner::UnnamedStruct {
                    fields: fields
                        .unnamed
                        .into_iter()
                        .enumerate()
                        .map(|(index, data)| {
                            let mut field =
                                Field::new(data, args.clone(), serde_args.clone(), &self_ty);
                            field.index = index;
                            field
                        })
                        .collect(),
                },
                Fields::Unit => {
                    panic!("Unit structs are not supported.");
                }
            },
            Data::Enum(data) => {
                let all_unit = data
                    .variants
                    .iter()
                    .all(|variant| matches!(variant.fields, Fields::Unit));
                let variants = data
                    .variants
                    .into_iter()
                    .map(|data| Variant::new(data, args.clone(), serde_args.clone(), &self_ty))
                    .collect::<Vec<_>>();

                if all_unit {
                    ContainerInner::UnitEnum { variants }
                } else {
                    ContainerInner::UnnamedEnum { variants }
                }
            }
            Data::Union(_) => {
                panic!("Unions are not supported.");
            }
        };

        let container = Self {
            args,
            attrs: input.attrs,
            generics: input.generics,
            ident: input.ident,
            inner,
            macro_type: ContainerMacro::Config,
            serde_args,
            vis: input.vis,
        };
        container.validate_args();
        container
    }

    fn validate_args(&self) {}

    pub fn get_partial_ident(&self) -> Ident {
        format_ident!("Partial{}", self.ident)
    }

    /// Return the name that identifies this container to users,
    /// which is used for schema references.
    pub fn get_name(&self) -> String {
        let dir = SerdeIoDirection::From;

        if let Some(name) = self.args.rename.as_ref().and_then(|rn| rn.get_name(dir)) {
            return name.into();
        }

        if let Some(name) = self
            .serde_args
            .rename
            .as_ref()
            .and_then(|rn| rn.get_name(dir))
        {
            return name.into();
        }

        self.ident.unraw().to_string()
    }

    /// Generics for a type *declaration*, which keeps bounds and defaults.
    /// The where clause is returned separately, as its position differs
    /// between named and tuple shapes.
    pub fn get_declaration_generics(&self) -> (&Generics, Option<&syn::WhereClause>) {
        (&self.generics, self.generics.where_clause.as_ref())
    }

    /// The same generics with a `'de` lifetime prepended, for a hand written
    /// `Deserialize` implementation.
    pub fn get_deserialize_generics(&self) -> Generics {
        let mut generics = self.generics.clone();

        generics.params.insert(
            0,
            syn::GenericParam::Lifetime(syn::LifetimeParam::new(syn::Lifetime::new(
                "'de",
                proc_macro2::Span::call_site(),
            ))),
        );

        generics
    }

    pub fn is_config_enum(&self) -> bool {
        matches!(self.macro_type, ContainerMacro::ConfigUnitEnum)
    }

    /// Return how the enum variants are tagged when serialized.
    pub fn get_tag_format(&self) -> SerdeTagFormat {
        if matches!(self.inner, ContainerInner::UnitEnum { .. }) || self.is_config_enum() {
            return SerdeTagFormat::Unit;
        }

        if self.serde_args.untagged {
            return SerdeTagFormat::Untagged;
        }

        match (&self.serde_args.tag, &self.serde_args.content) {
            (Some(tag), Some(content)) => {
                SerdeTagFormat::Adjacent(tag.to_owned(), content.to_owned())
            }
            (Some(tag), None) => SerdeTagFormat::Internal(tag.to_owned()),
            _ => SerdeTagFormat::External,
        }
    }

    /// Whether the partial enum is untagged, and requires
    /// each variant to be attempted in order when deserializing.
    pub fn is_untagged(&self) -> bool {
        matches!(
            self.inner,
            ContainerInner::UnnamedEnum { .. } | ContainerInner::UnitEnum { .. }
        ) && self.serde_args.untagged
    }

    /// Whether the partial enum is a literal union, like `"a" | "b" | boolean`.
    /// It's externally tagged, so a unit variant is written as its name, and
    /// any variant holding a value is untagged, so is written as that value
    /// alone. Serde's own failure to match such an enum doesn't say what it
    /// expected, so it's deserialized like an untagged enum instead.
    pub fn is_literal_union(&self) -> bool {
        let ContainerInner::UnnamedEnum { variants } = &self.inner else {
            return false;
        };

        matches!(self.get_tag_format(), SerdeTagFormat::External)
            && variants.iter().any(|variant| variant.is_untagged())
            && variants.iter().all(|variant| {
                variant.is_untagged()
                    || variant.is_unit_variant()
                    || variant.is_skipped_deserializing()
            })
    }

    /// Whether the partial enum implements `Deserialize` manually,
    /// rather than deriving it.
    fn has_manual_deserialize(&self) -> bool {
        self.is_untagged() || self.is_literal_union()
    }

    pub fn get_partial_attributes(&self) -> Vec<TokenStream> {
        let mut attrs = vec![];

        // Serde attributes come first, so that they take precedence
        // over any provided by the user via `partial(serde(...))`
        let serde_args = self.get_partial_serde_attribute_args();

        if !serde_args.is_empty() {
            attrs.push(quote! { #[serde(#serde_args)] });
        }

        // Inherit non-schematic attributes from the container,
        // like `doc`, `allow`, and `deprecated`
        for attr in &self.attrs {
            if is_inheritable_attribute(attr) {
                attrs.push(quote! { #attr });
            }
        }

        // Then apply any user-provided partial attributes, in the order
        // declared, as `partial(...)` may be repeated
        for partial in &self.args.partial {
            attrs.extend(partial.get_attributes());
        }

        attrs
    }

    /// `PartialConfig` requires the partial to be `DeserializeOwned`, which
    /// means every type argument must be too. Serde would otherwise infer a
    /// `T: Deserialize<'de>` bound of its own, and the two are ambiguous when
    /// both are in scope, so the bound is stated outright.
    ///
    /// Returns `None` when there is nothing generic to bound, or when the
    /// user supplied their own bound through `partial(serde(...))`.
    fn get_partial_deserialize_bound(&self) -> Option<String> {
        let params = self
            .generics
            .type_params()
            .map(|param| format!("{}: serde::de::DeserializeOwned", param.ident))
            .collect::<Vec<_>>();

        if params.is_empty() || self.has_partial_serde_bound() {
            return None;
        }

        Some(params.join(", "))
    }

    fn has_partial_serde_bound(&self) -> bool {
        self.args
            .partial
            .iter()
            .flat_map(PartialArg::get_attributes)
            .any(|attr| attr.to_string().contains("bound"))
    }

    /// Whether a struct accepts keys other than its fields, as serde does
    /// unless `deny_unknown_fields`. A config's partial denies them unless
    /// `#[config(allow_unknown_fields)]`, as that's what's deserialized.
    #[cfg(feature = "schema")]
    fn allows_unknown_fields(&self) -> bool {
        match self.macro_type {
            ContainerMacro::Config | ContainerMacro::ConfigUnitEnum => {
                !self.partial_denies_unknown_fields()
            }
            ContainerMacro::Schematic => !self.serde_args.deny_unknown_fields,
        }
    }

    /// The settings flattened into a named struct. Serde deserializes a struct
    /// with any as a map, so that each can read the keys left over.
    fn get_flattened_fields(&self) -> Vec<&Field> {
        match &self.inner {
            ContainerInner::NamedStruct { fields } => fields
                .iter()
                .filter(|field| field.is_flatten() && !field.is_skipped())
                .collect(),
            _ => vec![],
        }
    }

    /// Whether the partial denies keys that aren't its settings, which it does
    /// unless `#[config(allow_unknown_fields)]`. A flattened map collects every
    /// key the other settings don't, so none is unknown, and as serde reads them
    /// without taking them, denying them would reject every one of the map's.
    fn partial_denies_unknown_fields(&self) -> bool {
        if !matches!(self.inner, ContainerInner::NamedStruct { .. }) {
            return false;
        }

        if self.serde_args.deny_unknown_fields {
            return true;
        }

        !self.args.allow_unknown_fields
            && !self
                .get_flattened_fields()
                .iter()
                .any(|field| field.value.is_outer_map())
    }

    /// Panic on an attribute that parses but would do nothing for this derive,
    /// as a silent no-op is worse than a loud failure.
    pub fn validate_attributes(&self) {
        let variants = match &self.inner {
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                Some(variants)
            }
            _ => None,
        };

        match self.macro_type {
            ContainerMacro::Config => {
                if self.args.before_parse.is_some() {
                    panic!(
                        "Cannot use `before_parse` on a `Config`, as only a `ConfigEnum` parses a string. Derive `ConfigEnum` for an enum of unit variants."
                    );
                }

                let Some(variants) = variants else {
                    return;
                };

                self.validate_enum_attributes("Config");

                for variant in variants {
                    if variant.is_fallback() {
                        panic!(
                            "Cannot use `fallback` on a `Config` enum variant, as only a `ConfigEnum` falls back when parsing a string. Use `#[serde(untagged)]` on the variant for a value of another type."
                        );
                    }

                    if variant
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("default"))
                    {
                        panic!(
                            "Cannot use `#[default]` on a `Config` enum variant, as the derive implements `Default` itself. Use `#[setting(default)]` instead."
                        );
                    }
                }

                // A unit variant is written as a string, which is what its schema says
                if matches!(self.inner, ContainerInner::UnitEnum { .. })
                    && (self.serde_args.tag.is_some() || self.serde_args.untagged)
                {
                    panic!(
                        "Cannot use `tag` or `untagged` on a `Config` enum of only unit variants, as each is written as a string. Derive `ConfigEnum` for an enum of strings."
                    );
                }
            }
            ContainerMacro::ConfigUnitEnum => {
                self.validate_enum_attributes("ConfigEnum");
            }
            ContainerMacro::Schematic => {}
        }
    }

    fn validate_enum_attributes(&self, derive: &str) {
        if self.args.env_prefix.is_some() {
            panic!(
                "Cannot use `env_prefix` on a `{derive}` enum, as an enum reads no environment variables."
            );
        }

        if self.args.allow_unknown_fields {
            panic!(
                "Cannot use `allow_unknown_fields` on a `{derive}` enum, as only a struct has fields. Use it on a config that a variant holds instead."
            );
        }
    }

    /// Assert at compile time that a flattened config can be loaded. One that
    /// flattens a setting of its own is deserialized as a map, which reads the
    /// keys of the struct flattening it without taking them, so serde can only
    /// load it when neither denies unknown fields. A generic config is skipped,
    /// as a constant can't name its parameters.
    ///
    /// Also assert that a parent's `#[setting(nested, env_prefix)]` applies to
    /// something. It only prefixes the keys a nested config derives from its
    /// settings' names, which it only does with an `env_prefix` of its own.
    pub fn impl_partial_checks(&self) -> TokenStream {
        if self.generics.params.iter().next().is_some() {
            return quote! {};
        }

        let parent_allows = !self.partial_denies_unknown_fields();
        let container = self.ident.to_string();

        let checks = self
            .get_flattened_fields()
            .into_iter()
            .filter(|field| field.is_nested() && !field.value.is_collection())
            .filter_map(|field| {
                let nested = field.value.get_nested_type()?;
                let nested_name = to_type_string(nested.clone());
                let message = format!(
                    "`{container}.{}` flattens `{nested_name}`, which flattens a setting of its own. Serde can't deny unknown fields through it, so add `#[config(allow_unknown_fields)]` to both `{container}` and `{nested_name}`.",
                    field.get_name_original(),
                );
                let partial = quote! { <<#nested as schematic::Config>::Partial as schematic::PartialConfig> };

                Some(quote! {
                    const _: () = assert!(
                        !#partial::FLATTENS || (#parent_allows && #partial::ALLOWS_UNKNOWN_FIELDS),
                        #message
                    );
                })
            })
            .collect::<Vec<_>>();

        #[cfg(feature = "env")]
        let env_checks = self
            .inner
            .get_fields()
            .into_iter()
            .filter(|field| field.args.env_prefix.is_some())
            .filter_map(|field| {
                let nested = field.value.get_nested_type()?;
                let nested_name = to_type_string(nested.clone());
                let message = format!(
                    "`{container}.{}` sets an `env_prefix` for `{nested_name}`, which has no effect, as a parent's prefix only applies to the keys a config derives with an `env_prefix` of its own. Add `#[config(env_prefix)]` to `{nested_name}`, or remove it here.",
                    field.get_name_original(),
                );

                Some(quote! {
                    const _: () = assert!(
                        <<#nested as schematic::Config>::Partial as schematic::PartialConfig>::HAS_ENV_PREFIX,
                        #message
                    );
                })
            })
            .collect::<Vec<_>>();

        #[cfg(not(feature = "env"))]
        let env_checks: Vec<TokenStream> = vec![];

        quote! {
            #(#checks)*
            #(#env_checks)*
        }
    }

    pub fn get_partial_serde_attribute_args(&self) -> TokenStream {
        // The partial wraps every setting in `Option` and defaults them, which
        // serde won't make transparent, so it would still be written as a map
        if self.serde_args.transparent {
            panic!(
                "`#[serde(transparent)]` is not supported on a `Config`, as its partial is written as a map. Derive `Schematic` for a transparent type, and use it as a setting."
            );
        }

        let mut meta = vec![];

        match &self.inner {
            ContainerInner::NamedStruct { .. } => {
                meta.push(quote! { default });

                if self.partial_denies_unknown_fields() {
                    meta.push(quote! { deny_unknown_fields });
                }
            }
            ContainerInner::UnnamedStruct { .. } => {
                meta.push(quote! { default });
            }
            // Unit enums must remain externally tagged, otherwise
            // variants can only be deserialized from `null`
            ContainerInner::UnnamedEnum { .. } | ContainerInner::UnitEnum { .. } => {
                if let Some(tag) = &self.serde_args.tag {
                    meta.push(quote! { tag = #tag });
                }

                if let Some(content) = &self.serde_args.content {
                    meta.push(quote! { content = #content });
                }

                if self.serde_args.untagged {
                    meta.push(quote! { untagged });
                }
            }
        };

        if let Some(expecting) = &self.serde_args.expecting {
            meta.push(quote! { expecting = #expecting });
        }

        if let Some(bound) = self.get_partial_deserialize_bound() {
            meta.push(quote! { bound(deserialize = #bound) });
        }

        // Config attributes take precedence over serde attributes
        let renames = [
            ("rename", &self.args.rename, &self.serde_args.rename),
            (
                "rename_all",
                &self.args.rename_all,
                &self.serde_args.rename_all,
            ),
            (
                "rename_all_fields",
                &self.args.rename_all_fields,
                &self.serde_args.rename_all_fields,
            ),
        ];

        for (key, config_arg, serde_arg) in renames {
            if let Some(rename) = config_arg
                .as_ref()
                .or(serde_arg.as_ref())
                .filter(|rename| !rename.is_empty())
            {
                meta.push(rename.get_meta(key));
            }
        }

        quote! {
            #(#meta),*
        }
    }

    pub fn impl_full(&self) -> TokenStream {
        let base_name = &self.ident;
        let partial_name = self.get_partial_ident();

        let default_partial_method = self.impl_full_default_partial();
        let from_partial_method = self.impl_full_from_partial();
        let settings_method = self.impl_full_settings();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Config for #base_name #ty_generics #where_clause {
                type Partial = #partial_name #ty_generics;

                #default_partial_method
                #from_partial_method
                #settings_method
            }

            #[automatically_derived]
            impl #impl_generics Default for #base_name #ty_generics #where_clause {
                fn default() -> Self {
                    <Self as schematic::Config>::from_partial(
                        <Self as schematic::Config>::default_partial()
                    )
                }
            }
        }
    }

    /// Generate `default_partial`, which `Default` is built from, when a nested
    /// config's default partial isn't the same as its default values. A
    /// nested enum leaves the config its default variant holds to be applied
    /// when it's finalized, so each bare nested config takes its own default
    /// partial instead, which recurses into it.
    pub fn impl_full_default_partial(&self) -> TokenStream {
        let partial_name = self.get_partial_ident();

        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                let rows = fields
                    .iter()
                    .filter(|field| field.is_bare_nested())
                    .filter_map(|field| {
                        let key = field.get_key();
                        let ty = field.value.get_nested_type()?;

                        Some(quote! {
                            partial.#key = Some(<#ty as schematic::Config>::default_partial());
                        })
                    })
                    .collect::<Vec<_>>();

                if rows.is_empty() {
                    return quote! {};
                }

                quote! {
                    let mut partial = <Self::Partial as schematic::PartialConfig>::default_values(
                        &Default::default(),
                    )
                    .expect("Failed to generate default values.")
                    .unwrap_or_default();

                    #(#rows)*

                    partial
                }
            }
            ContainerInner::UnnamedEnum { variants } => {
                let Some(value) = variants
                    .iter()
                    .find(|variant| variant.is_default())
                    .and_then(|variant| variant.impl_full_default_partial(&partial_name))
                else {
                    return quote! {};
                };

                value
            }
            ContainerInner::UnitEnum { .. } => {
                return quote! {};
            }
        };

        quote! {
            fn default_partial() -> Self::Partial {
                #inner
            }
        }
    }

    pub fn impl_full_from_partial(&self) -> TokenStream {
        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } => {
                let mut rows = vec![];

                for field in fields {
                    let key = field.get_key();
                    let value = field.impl_full_from_partial().value;

                    rows.push(quote! {
                        #key: #value,
                    });
                }

                quote! {
                    Self {
                        #(#rows)*
                    }
                }
            }
            ContainerInner::UnnamedStruct { fields } => {
                let mut rows = vec![];

                for field in fields {
                    rows.push(field.impl_full_from_partial().value);
                }

                quote! {
                    Self(
                        #(#rows),*
                    )
                }
            }
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                let partial_name = self.get_partial_ident();
                let mut arms = vec![];

                for variant in variants {
                    arms.push(variant.impl_full_from_partial(&partial_name).value);
                }

                quote! {
                    match partial {
                        #(#arms)*
                    }
                }
            }
        };

        quote! {
            fn from_partial(partial: Self::Partial) -> Self {
                #inner
            }
        }
    }

    pub fn impl_full_settings(&self) -> TokenStream {
        let mut settings = vec![];

        match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                for field in fields {
                    let name = field.get_name_or_index();
                    let env_key = match field.get_env_var_name() {
                        Some(value) => quote! { .env(#value) },
                        None => quote! {},
                    };
                    let type_alias = to_type_string(field.value.ty.to_token_stream());
                    let setting = quote! {
                        ConfigSetting::new(#type_alias)
                            #env_key
                    };

                    // Nested settings are guarded, as a recursive config
                    // would otherwise never stop collecting them
                    let setting = match field.value.get_nested_type() {
                        Some(ty) => quote! {
                            schematic::internal::with_nested_settings::<#ty>(#setting)
                        },
                        None => setting,
                    };

                    settings.push(quote! {
                        (#name.into(), #setting),
                    });
                }
            }
            ContainerInner::UnnamedEnum { variants } => {
                for variant in variants {
                    let name = variant.get_name();
                    let type_alias = to_type_string(
                        variant
                            .values
                            .iter()
                            .map(|v| v.ty.to_token_stream())
                            .collect::<Vec<_>>()
                            .into_iter()
                            .collect::<TokenStream>(),
                    );

                    settings.push(quote! {
                        (#name.into(), ConfigSetting::new(#type_alias)),
                    });
                }
            }
            ContainerInner::UnitEnum { variants } => {
                for variant in variants {
                    let name = variant.get_name();

                    settings.push(quote! {
                        (#name.into(), ConfigSetting::new(#name)),
                    });
                }
            }
        };

        quote! {
            fn settings() -> schematic::ConfigSettingMap {
                use schematic::ConfigSetting;

                std::collections::BTreeMap::from_iter([
                    #(#settings)*
                ])
            }
        }
    }

    /// Generate the partial type declaration, in which every field
    /// is optional and nested configurations are replaced by their partials.
    pub fn impl_partial_type(&self) -> TokenStream {
        let partial_name = self.get_partial_ident();
        let attrs = self.get_partial_attributes();
        let vis = &self.vis;
        let (generics, where_clause) = self.get_declaration_generics();

        match &self.inner {
            ContainerInner::NamedStruct { fields } => quote! {
                #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
                #(#attrs)*
                #vis struct #partial_name #generics #where_clause {
                    #(#fields)*
                }
            },
            // A tuple struct takes its where clause after the fields
            ContainerInner::UnnamedStruct { fields } => quote! {
                #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
                #(#attrs)*
                #vis struct #partial_name #generics (
                    #(#fields)*
                ) #where_clause;
            },
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                // Untagged enums and literal unions implement `Deserialize`
                // manually, and all enums implement `Default` manually
                let derives = if self.has_manual_deserialize() {
                    quote! { Clone, Debug, PartialEq, serde::Serialize }
                } else {
                    quote! { Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize }
                };

                quote! {
                    #[derive(#derives)]
                    #(#attrs)*
                    #vis enum #partial_name #generics #where_clause {
                        #(#variants)*
                    }
                }
            }
        }
    }

    /// Generate a `Default` implementation for partial enums, as the
    /// derive is unable to determine the default variant. Partial structs
    /// derive `Default` instead.
    pub fn impl_partial_type_default(&self) -> TokenStream {
        let variants = match &self.inner {
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                variants
            }
            _ => return quote! {},
        };

        // Prefer the marked variant, otherwise fallback to the first
        let default_variant = variants
            .iter()
            .find(|variant| variant.is_default())
            .or_else(|| variants.first())
            .unwrap_or_else(|| panic!("Enums must have at least 1 variant."));

        let partial_name = self.get_partial_ident();
        let value = default_variant.impl_partial_default_value().value;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics Default for #partial_name #ty_generics #where_clause {
                fn default() -> Self {
                    Self::#value
                }
            }
        }
    }

    /// Generate a `Deserialize` implementation for untagged partial enums and
    /// literal unions, that attempts each variant in order, and reports all
    /// errors on failure, as the derived implementation loses this information.
    pub fn impl_partial_type_deserialize(&self) -> TokenStream {
        if !self.has_manual_deserialize() {
            return quote! {};
        }

        let partial_name = self.get_partial_ident();
        let literal_union = self.is_literal_union();
        let mut attempts = vec![];

        // Like serde, a literal union matches its named values before
        // attempting any untagged variant
        if literal_union {
            attempts.push(self.impl_partial_type_deserialize_literals(&partial_name));
        }

        for variant in self.inner.get_variants() {
            if literal_union && !variant.is_untagged() {
                continue;
            }

            let res = variant.impl_partial_type_deserialize(&partial_name);

            if !res.no_value {
                attempts.push(res.value);
            }
        }

        let de_generics = self.get_deserialize_generics();
        let (de_impl_generics, _, _) = de_generics.split_for_impl();
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #de_impl_generics serde::Deserialize<'de> for #partial_name #ty_generics #where_clause {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    use serde::de::Error as _;

                    // Buffer the content so that we can attempt to deserialize it multiple times
                    let content = schematic::internal::buffer_untagged_content(deserializer)?;
                    let mut errors: Vec<(&str, String)> = Vec::new();

                    #(#attempts)*

                    // All variants failed, so combine the errors into a single message
                    Err(D::Error::custom(schematic::internal::untagged_error_message(
                        stringify!(#partial_name),
                        &errors,
                    )))
                }
            }
        }
    }

    /// Generate a match of the buffered content against the name of every
    /// tagged unit variant in a literal union. The names fail together, as a
    /// single error listing each, rather than one error per variant.
    fn impl_partial_type_deserialize_literals(&self, partial_name: &Ident) -> TokenStream {
        let mut arms = vec![];
        let mut names = vec![];

        for variant in self.inner.get_variants() {
            if variant.is_untagged() {
                continue;
            }

            let res = variant.impl_partial_type_deserialize_literal(partial_name);

            if !res.no_value {
                arms.push(res.value);
                names.push(variant.get_name());
            }
        }

        if arms.is_empty() {
            return quote! {};
        }

        quote! {
            match schematic::internal::literal_content(&content) {
                #(#arms)*
                _ => errors.push(("", schematic::internal::literal_error_message(
                    &[#(#names),*],
                    &content,
                ))),
            }
        }
    }

    /// Generate `Schematic` implementations for both types, which are
    /// required by the `Config` and `PartialConfig` traits.
    pub fn impl_schematic(&self) -> TokenStream {
        let full = self.impl_schematic_full();
        let partial = self.impl_schematic_partial();

        quote! {
            #full
            #partial
        }
    }

    /// Generate the `ConfigEnum`, `FromStr`, `TryFrom`, and `Display`
    /// implementations for a unit-only enum.
    pub fn impl_config_enum(&self) -> TokenStream {
        let (ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants }) =
            &self.inner
        else {
            panic!("Only enums are supported.");
        };

        let input = format_ident!("input");
        let unknown = format_ident!("unknown");
        let mut values = vec![];
        let mut display_arms = vec![];
        let mut from_str_arms = vec![];
        let mut untagged_attempts = vec![];
        let mut fallback = None;

        for variant in variants {
            variant.validate_config_enum();

            values.push(variant.impl_config_enum_value());
            display_arms.push(variant.impl_config_enum_display());

            // A fallback absorbs anything left over, so it has to be matched
            // after every named value, and after every untagged variant
            if variant.is_fallback() {
                if fallback.is_some() {
                    panic!("Only 1 fallback variant is supported.");
                }

                fallback = Some(variant.impl_config_enum_fallback(&unknown));
            } else if variant.is_untagged() {
                untagged_attempts.push(variant.impl_config_enum_parse_untagged(&input));
            } else {
                from_str_arms.push(variant.impl_config_enum_from_str());
            }
        }

        // Without a fallback, an unknown value is an error
        let fallback = fallback.unwrap_or_else(|| {
            quote! {
                return Err(schematic::ConfigError::EnumUnknownVariant(#unknown.to_owned()));
            }
        });

        // Untagged variants parse the input as written, so it's kept
        // before `before_parse` normalizes it
        let keep_input = if untagged_attempts.is_empty() {
            quote! {}
        } else {
            quote! { let #input = value; }
        };

        let name = &self.ident;
        let before_parse = self.impl_config_enum_before_parse();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::ConfigEnum for #name #ty_generics #where_clause {
                fn variants() -> Vec<#name #ty_generics> {
                    vec![
                        #(#values),*
                    ]
                }
            }

            #[automatically_derived]
            impl #impl_generics std::str::FromStr for #name #ty_generics #where_clause {
                type Err = schematic::ConfigError;

                fn from_str(value: &str) -> std::result::Result<Self, schematic::ConfigError> {
                    #keep_input
                    #before_parse

                    Ok(match value {
                        #(#from_str_arms)*
                        #unknown => {
                            #(#untagged_attempts)*
                            #fallback
                        }
                    })
                }
            }

            #[automatically_derived]
            impl #impl_generics std::convert::TryFrom<String> for #name #ty_generics #where_clause {
                type Error = schematic::ConfigError;

                fn try_from(value: String) -> std::result::Result<Self, schematic::ConfigError> {
                    std::str::FromStr::from_str(&value)
                }
            }

            #[automatically_derived]
            impl #impl_generics std::convert::TryFrom<&String> for #name #ty_generics #where_clause {
                type Error = schematic::ConfigError;

                fn try_from(value: &String) -> std::result::Result<Self, schematic::ConfigError> {
                    std::str::FromStr::from_str(value)
                }
            }

            #[automatically_derived]
            impl #impl_generics std::convert::TryFrom<&str> for #name #ty_generics #where_clause {
                type Error = schematic::ConfigError;

                fn try_from(value: &str) -> std::result::Result<Self, schematic::ConfigError> {
                    std::str::FromStr::from_str(value)
                }
            }

            #[automatically_derived]
            impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_arms)*
                    }
                }
            }
        }
    }

    /// Normalize the incoming value before it is matched against a variant.
    /// Accepts the same case names as serde's `rename_all`.
    fn impl_config_enum_before_parse(&self) -> TokenStream {
        let Some(format) = self.args.before_parse.as_deref() else {
            return quote! {};
        };

        // Validated here so a typo fails the build rather than every parse
        validate_case_format("before_parse", format);

        quote! {
            let value = schematic::internal::format_case(value, #format);
            let value = value.as_str();
        }
    }

    /// Generate the `Schematic` implementation for the full type. This is
    /// also the whole of a standalone `#[derive(Schematic)]`, which has no
    /// partial type to pair with.
    #[cfg(not(feature = "schema"))]
    pub fn impl_schematic_full(&self) -> TokenStream {
        let base_name = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Schematic for #base_name #ty_generics #where_clause {}
        }
    }

    /// Generate the body of `schema_name`. A generic container appends the
    /// name of each type argument, since schemas are keyed by name alone and
    /// every instantiation would otherwise claim the same one.
    #[cfg(feature = "schema")]
    pub fn impl_schematic_name(&self) -> TokenStream {
        self.impl_schematic_name_for(self.get_name())
    }

    #[cfg(feature = "schema")]
    fn impl_schematic_name_for(&self, base_name_string: String) -> TokenStream {
        let params = self
            .generics
            .type_params()
            .map(|param| &param.ident)
            .collect::<Vec<_>>();

        if params.is_empty() {
            return quote! { Some(#base_name_string.into()) };
        }

        quote! {
            let mut name = String::from(#base_name_string);

            #(
                name.push_str(&schematic::schema::schema_name_of::<#params>());
            )*

            Some(name)
        }
    }

    /// Generate the `Schematic` implementation for the full type. This is
    /// also the whole of a standalone `#[derive(Schematic)]`, which has no
    /// partial type to pair with.
    #[cfg(feature = "schema")]
    pub fn impl_schematic_full(&self) -> TokenStream {
        let base_name = &self.ident;
        let inner = self.impl_schematic_type();
        let name = self.impl_schematic_name();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Schematic for #base_name #ty_generics #where_clause {
                fn schema_name() -> Option<String> {
                    #name
                }

                fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
                    use schematic::schema::*;

                    #inner
                }
            }
        }
    }

    /// Generate the `Schematic` implementation for the partial type, which
    /// derives its schema from the full type with all settings partialized.
    #[cfg(not(feature = "schema"))]
    pub fn impl_schematic_partial(&self) -> TokenStream {
        let partial_name = self.get_partial_ident();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Schematic for #partial_name #ty_generics #where_clause {}
        }
    }

    /// Generate the `Schematic` implementation for the partial type, which
    /// derives its schema from the full type with all settings partialized.
    #[cfg(feature = "schema")]
    pub fn impl_schematic_partial(&self) -> TokenStream {
        let base_name = &self.ident;
        let partial_name = self.get_partial_ident();
        // Named after the full type's schema, as a reference to it is
        // partialized into `Partial` + that name, which honours a rename
        let partial_schema_name =
            self.impl_schematic_name_for(format!("Partial{}", self.get_name()));
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Schematic for #partial_name #ty_generics #where_clause {
                fn schema_name() -> Option<String> {
                    #partial_schema_name
                }

                fn build_schema(schema: schematic::SchemaBuilder) -> schematic::Schema {
                    let mut schema = <#base_name #ty_generics as schematic::Schematic>::build_schema(schema);
                    schematic::internal::partialize_schema(&mut schema, true);
                    schema
                }
            }
        }
    }

    /// Generate the schema for the full configuration.
    #[cfg(feature = "schema")]
    pub fn impl_schematic_type(&self) -> TokenStream {
        use crate::utils::{extract_comment, extract_deprecated};

        let mut meta = vec![];

        if let Some(value) = extract_deprecated(&self.attrs) {
            meta.push(quote! { schema.set_deprecated(#value); });
        }

        if let Some(value) = extract_comment(&self.attrs) {
            meta.push(quote! { schema.set_description(#value); });
        }

        match &self.inner {
            // A skipped field is never read or written, so like an excluded
            // one it has no schema, and its type needn't implement `Schematic`
            ContainerInner::NamedStruct { fields } => {
                let fields = fields
                    .iter()
                    .filter(|field| !field.is_excluded() && !field.is_skipped())
                    .collect::<Vec<_>>();

                // A transparent struct is written as its one field
                if self.serde_args.transparent && fields.len() == 1 {
                    let inner = fields[0].impl_schema_type(false);

                    return quote! {
                        let mut schema = #inner;
                        #(#meta)*
                        schema
                    };
                }

                let types = fields
                    .iter()
                    .map(|field| field.impl_schema_type(true))
                    .collect::<Vec<_>>();

                let structure = if types.is_empty() {
                    quote! { StructType::default() }
                } else {
                    quote! {
                        StructType::new([
                            #(#types),*
                        ])
                    }
                };

                if self.allows_unknown_fields() {
                    quote! {
                        #(#meta)*
                        let mut structure = #structure;
                        structure.allow_unknown_fields = true;
                        schema.structure(structure)
                    }
                } else {
                    quote! {
                        #(#meta)*
                        schema.structure(#structure)
                    }
                }
            }
            ContainerInner::UnnamedStruct { fields } => {
                let types = fields
                    .iter()
                    .filter(|field| !field.is_excluded() && !field.is_skipped())
                    .map(|field| field.impl_schema_type(false))
                    .collect::<Vec<_>>();

                // A single value is transparent, so use its schema directly
                if types.len() == 1 {
                    let inner = &types[0];

                    quote! {
                        let mut schema = #inner;
                        #(#meta)*
                        schema
                    }
                } else {
                    quote! {
                        #(#meta)*
                        schema.tuple(TupleType::new([
                            #(#types),*
                        ]))
                    }
                }
            }
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                let unit =
                    matches!(self.inner, ContainerInner::UnitEnum { .. }) || self.is_config_enum();
                let tag_format = self.get_tag_format();
                let mut default_index = quote! { None };
                let mut types = vec![];
                let mut named_types = vec![];

                for variant in variants {
                    // Serde rejects a variant it never deserializes
                    if variant.is_excluded() || variant.is_skipped_deserializing() {
                        continue;
                    }

                    if variant.is_default() {
                        let index = types.len();

                        default_index = quote! { Some(#index) };
                    }

                    let name = variant.get_name();
                    let ty = variant.impl_schema_type(&tag_format);

                    named_types.push(quote! { (#name.into(), #ty) });
                    types.push(ty);
                }

                // Enums of only units are enumerable values,
                // otherwise they're a union of schemas. A union variant's
                // schema is its value alone, so the name travels beside it.
                let builder = if unit {
                    quote! { schema.enumerable(EnumType::from_schemas([#(#types),*], #default_index)) }
                } else {
                    quote! { schema.union(UnionType::from_named_schemas([#(#named_types),*], #default_index)) }
                };

                quote! {
                    #(#meta)*
                    #builder
                }
            }
        }
    }

    pub fn impl_partial(&self) -> TokenStream {
        let partial_name = self.get_partial_ident();
        let context = match self.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        let default_values_method = self.impl_partial_default_values();
        let env_values_method = self.impl_partial_env_values();
        let extends_from_method = self.impl_partial_extends_from();
        let finalize_method = self.impl_partial_finalize();
        let merge_method = self.impl_partial_merge();
        let validate_method = self.impl_partial_validate();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        // Only a named struct can be flattened into another
        let mut consts = vec![];

        if matches!(self.inner, ContainerInner::NamedStruct { .. }) {
            if !self.get_flattened_fields().is_empty() {
                consts.push(quote! { const FLATTENS: bool = true; });
            }

            if !self.partial_denies_unknown_fields() {
                consts.push(quote! { const ALLOWS_UNKNOWN_FIELDS: bool = true; });
            }

            if self.args.env_prefix.is_some() {
                consts.push(quote! { const HAS_ENV_PREFIX: bool = true; });
            }
        }

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::PartialConfig for #partial_name #ty_generics #where_clause {
                type Context = #context;

                #(#consts)*

                #default_values_method
                #env_values_method
                #extends_from_method
                #finalize_method
                #merge_method
                #validate_method
            }
        }
    }

    pub fn impl_partial_default_values(&self) -> TokenStream {
        let mut requires_internal = false;

        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } => {
                let mut rows = vec![];

                for field in fields {
                    let res = field.impl_partial_default_value();

                    if !res.no_value {
                        let key = field.get_key();
                        let value = res.value;

                        rows.push(quote! {
                            #key: #value,
                        });
                    }

                    if res.requires_internal {
                        requires_internal = true;
                    }
                }

                // Do not implement method
                if rows.is_empty() {
                    return quote! {};
                }

                let default_row = ImplResult::impl_struct_default(rows.len() != fields.len());

                quote! {
                    Ok(Some(Self {
                        #(#rows)*
                        #default_row
                    }))
                }
            }
            ContainerInner::UnnamedStruct { fields } => {
                let mut rows = vec![];
                let mut all_none = true;

                for field in fields {
                    let res = field.impl_partial_default_value();

                    if res.no_value {
                        rows.push(quote! { None });
                    } else {
                        all_none = false;
                        let value = res.value;

                        rows.push(quote! {
                            #value
                        });
                    }

                    if res.requires_internal {
                        requires_internal = true;
                    }
                }

                // Do not implement method
                if all_none {
                    return quote! {};
                }

                quote! {
                    Ok(Some(Self(
                        #(#rows),*
                    )))
                }
            }
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                let default_variants = variants
                    .iter()
                    .filter(|v| v.is_default())
                    .collect::<Vec<_>>();

                if default_variants.len() > 1 {
                    panic!("Only 1 variant may be marked as default.");
                }

                match default_variants.first() {
                    Some(default_variant) => {
                        let res = default_variant.impl_partial_default_values();

                        if res.requires_internal {
                            requires_internal = true;
                        }

                        if res.no_value {
                            quote! {
                                Ok(None)
                            }
                        } else {
                            let value = res.value;

                            quote! {
                                Ok(Some(Self::#value))
                            }
                        }
                    }
                    None => quote! {
                        Ok(None)
                    },
                }
            }
        };

        let internal = ImplResult::impl_use_internal(requires_internal);

        quote! {
            fn default_values(context: &Self::Context) -> std::result::Result<Option<Self>, schematic::ConfigError> {
                #internal
                #inner
            }
        }
    }

    #[cfg(not(feature = "env"))]
    pub fn impl_partial_env_values(&self) -> TokenStream {
        quote! {}
    }

    #[cfg(feature = "env")]
    pub fn impl_partial_env_values(&self) -> TokenStream {
        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                let mut rows = vec![];

                for field in fields {
                    let res = field.impl_partial_env_value();

                    if !res.no_value {
                        let key = field.get_key();
                        let value = res.value;

                        rows.push(quote! {
                            partial.#key = #value;
                        });
                    }
                }

                // Do not implement method
                if rows.is_empty() {
                    return quote! {};
                }

                quote! {
                    #(#rows)*
                }
            }

            // Enums don't support env vars
            _ => return quote! {},
        };

        let internal = ImplResult::impl_use_internal(true);

        // A prefix passed in by a parent is read alongside the container's
        // own, rather than replacing it, so that the keys reported in the
        // schema and `settings()` always work
        let own_prefix = if let Some(env_prefix) = &self.args.env_prefix {
            if env_prefix.is_empty() {
                panic!("Attribute `env_prefix` cannot be empty.");
            }

            quote! { Some(#env_prefix) }
        } else {
            quote! { None }
        };

        // Both are implemented, as the trait's `env_values_with_prefix` falls
        // back to `env_values`, for a hand-written partial
        quote! {
            fn env_values() -> std::result::Result<Option<Self>, schematic::ConfigError> {
                Self::env_values_with_prefix(None)
            }

            fn env_values_with_prefix(prefix: Option<&str>) -> std::result::Result<Option<Self>, schematic::ConfigError> {
                #internal

                let mut env = EnvManager::new(prefix, #own_prefix);
                let mut partial = Self::default();

                #inner

                Ok(if env.is_empty() {
                    None
                } else {
                    Some(partial)
                })
            }
        }
    }

    #[cfg(not(feature = "extends"))]
    pub fn impl_partial_extends_from(&self) -> TokenStream {
        quote! {}
    }

    #[cfg(feature = "extends")]
    pub fn impl_partial_extends_from(&self) -> TokenStream {
        let extendable = self
            .inner
            .get_fields()
            .into_iter()
            .filter(|field| field.is_extendable())
            .collect::<Vec<_>>();

        // Do not implement method
        if extendable.is_empty() {
            return quote! {};
        }

        if !matches!(self.inner, ContainerInner::NamedStruct { .. }) {
            panic!("Only named structs can use `extend` settings.");
        }

        if extendable.len() > 1 {
            let names = extendable
                .iter()
                .map(|field| field.get_name_original())
                .collect::<Vec<_>>();

            panic!(
                "Only 1 setting may use `extend`, found: {}",
                names.join(", ")
            );
        }

        let res = extendable[0].impl_partial_extends_from();
        let inner = if res.no_value {
            quote! { None }
        } else {
            res.value
        };

        quote! {
            fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                #inner
            }
        }
    }

    pub fn impl_partial_finalize(&self) -> TokenStream {
        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                let mut statements = vec![];

                for field in fields {
                    let res = field.impl_partial_finalize();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                // Default values and environment variables are layers the loader
                // merges, and those of nested configs are part of this config's
                // own, so only transforms and nested configs remain
                if statements.is_empty() {
                    return quote! {};
                }

                return quote! {
                    fn finalize(self, context: &Self::Context) -> std::result::Result<Self, schematic::ConfigError> {
                        let mut partial = self;

                        #(#statements)*

                        Ok(partial)
                    }
                };
            }
            ContainerInner::UnnamedEnum { variants } => {
                let mut statements = vec![];

                for variant in variants {
                    let res = variant.impl_partial_finalize();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                if statements.is_empty() {
                    quote! {
                        Ok(self)
                    }
                } else {
                    quote! {
                        Ok(match self {
                            #(#statements)*
                            _ => self
                        })
                    }
                }
            }
            ContainerInner::UnitEnum { .. } => {
                return quote! {};
            }
        };

        quote! {
            fn finalize(self, context: &Self::Context) -> std::result::Result<Self, schematic::ConfigError> {
                #inner
            }
        }
    }

    pub fn impl_partial_merge(&self) -> TokenStream {
        match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                let mut statements = vec![];

                for field in fields {
                    let res = field.impl_partial_merge();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                if statements.is_empty() {
                    return quote! {};
                }

                let internal = ImplResult::impl_use_internal(true);

                quote! {
                    fn merge(
                        &mut self,
                        context: &Self::Context,
                        mut next: Self,
                    ) -> std::result::Result<(), schematic::ConfigError> {
                        #internal

                        MergeManager::new(context)
                        #(#statements)*;

                        Ok(())
                    }
                }
            }
            ContainerInner::UnnamedEnum { variants } | ContainerInner::UnitEnum { variants } => {
                let mut statements = vec![];
                let mut requires_internal = false;

                for variant in variants {
                    let res = variant.impl_partial_merge();

                    if !res.no_value {
                        statements.push(res.value);
                    }

                    if res.requires_internal {
                        requires_internal = true;
                    }
                }

                let internal = ImplResult::impl_use_internal(requires_internal);
                let inner = if statements.is_empty() {
                    quote! {
                        *self = next;
                    }
                } else {
                    quote! {
                        match self {
                            #(#statements)*
                            _ => {
                                *self = next;
                            }
                        };
                    }
                };

                quote! {
                    fn merge(
                        &mut self,
                        context: &Self::Context,
                        mut next: Self,
                    ) -> std::result::Result<(), schematic::ConfigError> {
                        #internal
                        #inner
                        Ok(())
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "validate"))]
    pub fn impl_partial_validate(&self) -> TokenStream {
        quote! {}
    }

    #[cfg(feature = "validate")]
    pub fn impl_partial_validate(&self) -> TokenStream {
        let inner = match &self.inner {
            ContainerInner::NamedStruct { fields } | ContainerInner::UnnamedStruct { fields } => {
                let mut statements = vec![];

                for field in fields {
                    let res = field.impl_partial_validate();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                if statements.is_empty() {
                    return quote! {};
                }

                quote! {
                     #(#statements)*
                }
            }
            ContainerInner::UnnamedEnum { variants } => {
                let mut statements = vec![];

                for variant in variants {
                    let res = variant.impl_partial_validate();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                if statements.is_empty() {
                    return quote! {};
                }

                quote! {
                    match self {
                        #(#statements)*
                        _ => {}
                    };
                }
            }
            ContainerInner::UnitEnum { .. } => {
                return quote! {};
            }
        };

        let internal = ImplResult::impl_use_internal(true);

        quote! {
            fn validate_with_path(
                &self,
                context: &Self::Context,
                finalizing: bool,
                path: schematic::Path
            ) -> std::result::Result<(), Vec<schematic::ValidateError>> {
                #internal

                let mut validate = ValidateManager::new(context, finalizing, path);
                #inner

                if !validate.errors.is_empty() {
                    return Err(validate.errors);
                }

                Ok(())
            }
        }
    }
}

// #[derive(Config)]
impl ToTokens for Container {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.validate_attributes();

        match self.macro_type {
            ContainerMacro::Config => {
                // Partial type
                tokens.extend(self.impl_partial_type());
                tokens.extend(self.impl_partial_type_default());
                tokens.extend(self.impl_partial_type_deserialize());
                tokens.extend(self.impl_partial());
                tokens.extend(self.impl_partial_checks());

                // Full type
                tokens.extend(self.impl_full());

                // Both types
                tokens.extend(self.impl_schematic());
            }
            ContainerMacro::ConfigUnitEnum => {
                tokens.extend(self.impl_config_enum());
                tokens.extend(self.impl_schematic_full());
            }
            ContainerMacro::Schematic => {
                tokens.extend(self.impl_schematic_full());
            }
        }
    }
}

#[derive(Debug)]
pub enum ContainerInner {
    NamedStruct { fields: Vec<Field> },
    UnnamedStruct { fields: Vec<Field> },
    // TODO: NamedEnum
    UnnamedEnum { variants: Vec<Variant> },
    UnitEnum { variants: Vec<Variant> },
}

impl ContainerInner {
    pub fn get_fields(&self) -> Vec<&Field> {
        match self {
            Self::NamedStruct { fields } | Self::UnnamedStruct { fields } => {
                fields.iter().collect()
            }
            _ => vec![],
        }
    }

    pub fn get_variants(&self) -> Vec<&Variant> {
        match self {
            Self::UnnamedEnum { variants } | Self::UnitEnum { variants } => {
                variants.iter().collect()
            }
            _ => vec![],
        }
    }
}
