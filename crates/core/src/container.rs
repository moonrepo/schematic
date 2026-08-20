use crate::args::{
    PartialArg, SerdeContainerArgs, SerdeIoDirection, SerdeRenameArg, SerdeTagFormat,
};
use crate::field::{EnvKey, Field};
use crate::utils::{ImplResult, is_inheritable_attribute, to_type_string, validate_case_format};
use crate::variant::Variant;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields, Generics, Ident, Visibility};

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
    pub partial: Option<PartialArg>,

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

        let inner = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => ContainerInner::NamedStruct {
                    fields: fields
                        .named
                        .into_iter()
                        .map(|data| Field::new(data, args.clone(), serde_args.clone()))
                        .collect(),
                },
                Fields::Unnamed(fields) => ContainerInner::UnnamedStruct {
                    fields: fields
                        .unnamed
                        .into_iter()
                        .enumerate()
                        .map(|(index, data)| {
                            let mut field = Field::new(data, args.clone(), serde_args.clone());
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
                    .map(|data| Variant::new(data, args.clone(), serde_args.clone()))
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

        self.ident.to_string()
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

        // Then apply any user-provided partial attributes
        if let Some(partial) = &self.args.partial {
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
        self.args.partial.as_ref().is_some_and(|partial| {
            partial
                .get_attributes()
                .iter()
                .any(|attr| attr.to_string().contains("bound"))
        })
    }

    pub fn get_partial_serde_attribute_args(&self) -> TokenStream {
        let mut meta = vec![];

        match &self.inner {
            ContainerInner::NamedStruct { .. } => {
                meta.push(quote! { default });

                if self.serde_args.deny_unknown_fields || !self.args.allow_unknown_fields {
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

        let from_partial_method = self.impl_full_from_partial();
        let settings_method = self.impl_full_settings();
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Config for #base_name #ty_generics #where_clause {
                type Partial = #partial_name #ty_generics;

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
                    // Only explicit keys are known statically, as derived
                    // keys depend on the prefix in effect at runtime
                    let env_key = match field.get_env_var() {
                        Some(EnvKey::Explicit(value)) => quote! { .env(#value) },
                        _ => quote! {},
                    };
                    let nested = if field.is_nested() {
                        let value = field.value.get_inner_type();
                        quote! { .nested(#value::settings()) }
                    } else {
                        quote! {}
                    };
                    let type_alias = to_type_string(field.value.ty.to_token_stream());

                    settings.push(quote! {
                        (#name.into(), ConfigSetting::new(#type_alias)
                            #env_key
                            #nested
                        ),
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
                // Untagged enums implement `Deserialize` manually,
                // and all enums implement `Default` manually
                let derives = if self.is_untagged() {
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

    /// Generate a `Deserialize` implementation for untagged partial enums,
    /// that attempts each variant in order, and reports all errors on failure,
    /// as the derived implementation loses this information.
    pub fn impl_partial_type_deserialize(&self) -> TokenStream {
        if !self.is_untagged() {
            return quote! {};
        }

        let partial_name = self.get_partial_ident();
        let mut attempts = vec![];

        for variant in self.inner.get_variants() {
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
                    let content = deserializer.deserialize_any(schematic::serde_content::ValueVisitor)?;
                    let mut errors: Vec<(&str, String)> = Vec::new();

                    #(#attempts)*

                    // All variants failed, so combine the errors into a single message
                    let mut message = format!(
                        "failed to parse as any variant of {}:",
                        stringify!(#partial_name)
                    );

                    for (variant, error) in &errors {
                        message.push_str(&format!("\n- {variant}: {error}"));
                    }

                    Err(D::Error::custom(message))
                }
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

        let mut values = vec![];
        let mut display_arms = vec![];
        let mut from_str_arms = vec![];
        let mut fallback_arm = None;

        for variant in variants {
            variant.validate_config_enum();

            values.push(variant.impl_config_enum_value());
            display_arms.push(variant.impl_config_enum_display());

            // A fallback absorbs anything left over, so it has to be matched
            // after every named value
            if variant.is_fallback() {
                if fallback_arm.is_some() {
                    panic!("Only 1 fallback variant is supported.");
                }

                fallback_arm = Some(variant.impl_config_enum_from_str());
            } else {
                from_str_arms.push(variant.impl_config_enum_from_str());
            }
        }

        // Without a fallback, an unknown value is an error
        let fallback_arm = fallback_arm.unwrap_or_else(|| {
            quote! {
                unknown => {
                    return Err(schematic::ConfigError::EnumUnknownVariant(unknown.to_owned()));
                }
            }
        });

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
                    #before_parse

                    Ok(match value {
                        #(#from_str_arms)*
                        #fallback_arm
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
        let partial_schema_name = self.impl_schematic_name_for(partial_name.to_string());
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
            ContainerInner::NamedStruct { fields } => {
                let types = fields
                    .iter()
                    .filter(|field| !field.is_excluded())
                    .map(|field| field.impl_schema_type(true))
                    .collect::<Vec<_>>();

                if types.is_empty() {
                    quote! {
                        #(#meta)*
                        schema.structure(StructType::default())
                    }
                } else {
                    quote! {
                        #(#meta)*
                        schema.structure(StructType::new([
                            #(#types),*
                        ]))
                    }
                }
            }
            ContainerInner::UnnamedStruct { fields } => {
                let types = fields
                    .iter()
                    .filter(|field| !field.is_excluded())
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

                for variant in variants {
                    if variant.is_excluded() {
                        continue;
                    }

                    if variant.is_default() {
                        let index = types.len();

                        default_index = quote! { Some(#index) };
                    }

                    types.push(variant.impl_schema_type(&tag_format));
                }

                // Enums of only units are enumerable values,
                // otherwise they're a union of schemas
                let builder = if unit {
                    quote! { schema.enumerable(EnumType::from_schemas([#(#types),*], #default_index)) }
                } else {
                    quote! { schema.union(UnionType::from_schemas([#(#types),*], #default_index)) }
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

        quote! {
            #[automatically_derived]
            impl #impl_generics schematic::PartialConfig for #partial_name #ty_generics #where_clause {
                type Context = #context;

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
                        let res = default_variant.impl_partial_default_value();

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

        let prefix_fallback = if let Some(env_prefix) = &self.args.env_prefix {
            if env_prefix.is_empty() {
                panic!("Attribute `env_prefix` cannot be empty.");
            }

            quote! { prefix.or(Some(#env_prefix)) }
        } else {
            quote! { prefix }
        };

        quote! {
            fn env_values_with_prefix(prefix: Option<&str>) -> std::result::Result<Option<Self>, schematic::ConfigError> {
                #internal

                let mut env = EnvManager::new(#prefix_fallback);
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
                .map(|field| field.get_name_original().to_string())
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

                #[cfg(feature = "env")]
                {
                    statements.push(quote! {
                        if let Some(layer) = Self::env_values()? {
                            partial.merge(context, layer)?;
                        }
                    });
                }

                for field in fields {
                    let res = field.impl_partial_finalize();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

                quote! {
                    let mut partial = Self::default();

                    if let Some(layer) = Self::default_values(context)? {
                        partial.merge(context, layer)?;
                    }

                    partial.merge(context, self)?;

                    #(#statements)*

                    Ok(partial)
                }
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
        match self.macro_type {
            ContainerMacro::Config => {
                // Partial type
                tokens.extend(self.impl_partial_type());
                tokens.extend(self.impl_partial_type_default());
                tokens.extend(self.impl_partial_type_deserialize());
                tokens.extend(self.impl_partial());

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
