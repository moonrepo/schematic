use crate::args::{PartialArg, SerdeContainerArgs, SerdeRenameArg};
use crate::field::Field;
use crate::utils::{ImplResult, is_inheritable_attribute};
use crate::variant::Variant;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields, Ident, Visibility};

// #[config()], #[schematic()]
#[derive(Debug, Default, FromDeriveInput)]
#[darling(default, attributes(config, schematic), supports(struct_any, enum_any))]
pub struct ContainerArgs {
    // config
    pub allow_unknown_fields: bool,
    pub context: Option<ExprPath>,
    #[cfg(feature = "env")]
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

    // inherited
    pub attrs: Vec<Attribute>,
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
            ident: input.ident,
            inner,
            serde_args,
            vis: input.vis,
        };
        container.validate_args();
        container
    }

    fn validate_args(&self) {}

    pub fn get_partial_attributes(&self) -> Vec<TokenStream> {
        let serde_args = self.get_partial_serde_attribute_args();
        let mut attrs = vec![quote! { #[serde(#serde_args) ]}];

        for attr in &self.attrs {
            if is_inheritable_attribute(attr) {
                attrs.push(quote! { #attr });
            }
        }

        // TODO
        // let partial = &self.args.partial;
        // attrs.push(quote! { #partial });

        attrs
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
            ContainerInner::UnnamedEnum { .. } => {
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
            ContainerInner::UnitEnum { .. } => {
                meta.push(quote! { untagged });
            }
        };

        if let Some(expecting) = &self.serde_args.expecting {
            meta.push(quote! { expecting = #expecting });
        }

        if let Some(rename) = &self.serde_args.rename {
            meta.push(rename.get_meta("rename"));
        }

        if let Some(rename_all) = &self.serde_args.rename_all {
            meta.push(rename_all.get_meta("rename_all"));
        }

        if let Some(rename_all_fields) = &self.serde_args.rename_all_fields {
            meta.push(rename_all_fields.get_meta("rename_all_fields"));
        }

        quote! {
            #(#meta),*
        }
    }

    pub fn impl_partial(&self) -> TokenStream {
        let base_name = &self.ident;
        let partial_name = format_ident!("Partial{base_name}");
        let context = match self.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        let default_values_method = self.impl_partial_default_values();
        let env_values_method = self.impl_partial_env_values();
        let extends_from_method = self.impl_partial_extends_from();
        let merge_method = self.impl_partial_merge();
        let validate_method = self.impl_partial_validate();

        quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                type Context = #context;

                #default_values_method
                #env_values_method
                #extends_from_method
                #merge_method
                #validate_method
            }

            #[automatically_derived]
            impl schematic::Config for #base_name {
                // TODO
            }

            #[automatically_derived]
            impl Default for #base_name {
                fn default() -> Self {
                    <Self as schematic::Config>::from_partial(
                        <Self as schematic::Config>::default_partial()
                    )
                }
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

            quote! { prefix.or_else(Some(#env_prefix)) }
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
        if let ContainerInner::NamedStruct { fields } = &self.inner {
            let mut names = vec![];
            let mut inner = quote! { None };

            for field in fields {
                if field.is_extendable() {
                    names.push(field.get_name_original().to_string());

                    let res = field.impl_partial_extends_from();

                    if !res.no_value {
                        inner = res.value;
                    }
                }
            }

            if names.len() > 1 {
                panic!(
                    "Only 1 setting may use `extend`, found: {}",
                    names.join(", ")
                );
            }

            quote! {
                fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                    #inner
                }
            }
        } else {
            panic!("Only named structs can use `extend` settings.");
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

                for variant in variants {
                    let res = variant.impl_partial_merge();

                    if !res.no_value {
                        statements.push(res.value);
                    }
                }

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
                finalize: bool,
                path: schematic::Path
            ) -> std::result::Result<(), Vec<schematic::ValidateError>> {
                #internal

                let mut validate = ValidateManager::new(context, finalize, path);
                #inner

                if !validate.errors.is_empty() {
                    return Err(validate.errors);
                }

                Ok(())
            }
        }
    }
}

impl ToTokens for Container {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // TODO
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
