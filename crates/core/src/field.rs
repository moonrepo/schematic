use crate::args::{NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg};
use crate::container::ContainerArgs;
use crate::field_value::FieldValue;
use crate::utils::{
    ImplResult, format_case, get_renamed_value, is_inheritable_attribute, preserve_str_literal,
};
use darling::FromAttributes;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, Expr, ExprPath, Field as NativeField, FieldModifiers, Ident, Visibility};

// #[schema()], #[setting()]
#[derive(Debug, FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {
    #[darling(with = preserve_str_literal, map = "Some")]
    pub default: Option<Expr>,
    pub env: Option<String>,
    pub env_prefix: Option<String>,
    pub exclude: bool,
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<NestedArg>,
    pub parse_env: Option<ExprPath>,
    pub partial: Option<PartialArg>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    pub validate: Option<crate::args::ValidateArg>,

    // serde
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub flatten: bool,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_deserializing_if: Option<String>,
    pub skip_serializing: bool,
    pub skip_serializing_if: Option<String>,
}

/// The environment variable key for a setting.
#[derive(Debug, PartialEq)]
pub enum EnvKey {
    /// An explicit key from `#[setting(env)]`, that is used as-is
    /// and takes precedence over any prefix.
    Explicit(String),
    /// A key derived from the setting name when using `env_prefix`,
    /// that is prefixed at runtime.
    Derived(String),
}

impl EnvKey {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Explicit(key) | Self::Derived(key) => key,
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub value: FieldValue,

    // args
    pub args: FieldArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Option<Ident>, // Named
    pub index: usize,         // Unnamed
    pub modifiers: FieldModifiers,
    pub vis: Visibility,
}

impl Field {
    pub fn new(
        field: NativeField,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Self {
        let args = FieldArgs::from_attributes(&field.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&field.attrs).unwrap();

        let field = Self {
            attrs: field.attrs,
            container_args,
            ident: field.ident,
            index: 0,
            modifiers: field.modifiers,
            serde_args,
            serde_container_args,
            vis: field.vis,
            value: FieldValue::new(field.ty, args.nested.as_ref()),
            args,
        };

        // dbg!(&field);

        field.validate_args();
        field
    }

    fn validate_args(&self) {
        #[cfg(feature = "env")]
        {
            if self.args.env_prefix.is_some() && self.args.nested.is_none() {
                panic!("Cannot use `env_prefix` without `nested`.");
            }

            if self.args.parse_env.is_some() && self.args.env.is_none() {
                panic!("Cannot use `parse_env` without `env`.");
            }
        }

        if self.is_required() && !self.value.is_outer_option_wrapped() {
            panic!("Cannot use `required` with non-optional settings.");
        }
    }

    #[cfg(not(feature = "env"))]
    pub fn get_env_var(&self) -> Option<EnvKey> {
        None
    }

    #[cfg(feature = "env")]
    pub fn get_env_var(&self) -> Option<EnvKey> {
        if self.args.env.is_some() && self.args.env_prefix.is_some() {
            panic!("Cannot use `env` and `env_prefix` together.");
        }

        if let Some(env_key) = &self.args.env {
            if env_key.is_empty() {
                panic!("Attribute `env` cannot be empty.");
            }

            if self.is_nested() {
                panic!("Cannot use `env` with `nested`, use `env_prefix` instead?");
            }

            return Some(EnvKey::Explicit(env_key.to_owned()));
        }

        // Otherwise derive a key from the setting name, but only when this
        // container declares a prefix, as that's how a setting opts into
        // being sourced from the environment.
        //
        // Unnamed settings have no name to derive from, so they may only
        // be sourced with an explicit `env`.
        if self.container_args.env_prefix.is_some() && self.ident.is_some() {
            // Deliberately un-cased: `rename_all` shapes the serialized key,
            // while the variable name stays derived from the Rust name (or
            // an explicit rename).
            let name = self
                .get_name_renamed()
                .unwrap_or_else(|| self.get_name_original().to_string());

            return Some(EnvKey::Derived(name.to_uppercase()));
        }

        None
    }

    pub fn get_key(&self) -> TokenStream {
        self.ident
            .as_ref()
            .map(|name| quote! { #name })
            .unwrap_or_else(|| {
                let index = Index(self.index);

                quote! { #index }
            })
    }

    /// Return an explicit `rename`, if one was provided. Casing is not
    /// applied, as a rename is used exactly as written.
    fn get_name_renamed(&self) -> Option<String> {
        get_renamed_value(self.args.rename.as_ref(), self.serde_args.rename.as_ref())
    }

    /// Return the name this setting serializes as. A container `rename_all`
    /// is applied here, and not only forwarded to serde, so that schemas,
    /// `settings()`, and validation paths describe the key that is actually
    /// accepted. There is no default case.
    pub fn get_name(&self) -> String {
        if let Some(name) = self.get_name_renamed() {
            return name;
        }

        let name = self.get_name_original().to_string();

        match get_renamed_value(
            self.container_args.rename_all.as_ref(),
            self.serde_container_args.rename_all.as_ref(),
        ) {
            Some(format) => format_case(&format, &name, false),
            None => name,
        }
    }

    pub fn get_name_original(&self) -> &Ident {
        self.ident
            .as_ref()
            .expect("Name only usable on named fields!")
    }

    /// Return the name that identifies this setting to users, which is the
    /// serde name for named settings, and the position for unnamed ones.
    pub fn get_name_or_index(&self) -> String {
        if self.ident.is_some() {
            self.get_name()
        } else {
            self.index.to_string()
        }
    }

    pub fn is_excluded(&self) -> bool {
        #[cfg(feature = "schema")]
        {
            self.args.exclude
        }

        #[cfg(not(feature = "schema"))]
        {
            false
        }
    }

    pub fn is_extendable(&self) -> bool {
        #[cfg(feature = "extends")]
        {
            self.args.extend
        }

        #[cfg(not(feature = "extends"))]
        {
            false
        }
    }

    pub fn is_flatten(&self) -> bool {
        self.args.flatten || self.serde_args.flatten
    }

    pub fn is_nested(&self) -> bool {
        self.args
            .nested
            .as_ref()
            .is_some_and(|nested| nested.is_nested())
    }

    /// Whether the setting accepts a `null` value.
    pub fn is_nullable(&self) -> bool {
        self.value.is_outer_option_wrapped()
    }

    /// Whether the setting can be omitted, because a default is provided.
    pub fn is_optional(&self) -> bool {
        self.args.default.is_some() || self.serde_args.default
    }

    pub fn is_required(&self) -> bool {
        self.args.required
    }

    pub fn is_skipped(&self) -> bool {
        self.args.skip || self.serde_args.skip
    }

    pub fn get_aliases(&self) -> Vec<&String> {
        let mut aliases = vec![];

        for alias in self.args.alias.iter().chain(self.serde_args.alias.iter()) {
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }

        aliases
    }
}

impl Field {
    pub fn get_partial_attributes(&self) -> Vec<TokenStream> {
        let mut attrs = vec![];

        // Serde attributes come first, so that they take precedence
        // over any provided by the user via `partial(serde(...))`
        let serde_args = self.get_partial_serde_attribute_args();

        if !serde_args.is_empty() {
            attrs.push(quote! { #[serde(#serde_args)] });
        }

        // Inherit non-schematic attributes from the field,
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

    pub fn get_partial_serde_attribute_args(&self) -> TokenStream {
        let mut meta = vec![];

        // Aliases can be provided by both, so combine them
        let mut aliases: Vec<&String> = vec![];

        for alias in self.args.alias.iter().chain(self.serde_args.alias.iter()) {
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }

        for alias in aliases {
            meta.push(quote! { alias = #alias });
        }

        if self.args.flatten || self.serde_args.flatten {
            meta.push(quote! { flatten });
        }

        // Setting attributes take precedence over serde attributes
        if let Some(rename) = self
            .args
            .rename
            .as_ref()
            .or(self.serde_args.rename.as_ref())
            .filter(|rename| !rename.is_empty())
        {
            meta.push(rename.get_meta("rename"));
        }

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
        } else {
            if self.args.skip_serializing || self.serde_args.skip_serializing {
                meta.push(quote! { skip_serializing });
            } else if let Some(func) = self
                .args
                .skip_serializing_if
                .as_ref()
                .or(self.serde_args.skip_serializing_if.as_ref())
            {
                meta.push(quote! { skip_serializing_if = #func });
            } else {
                // Partial values are always optional, so avoid serializing `None`
                meta.push(quote! { skip_serializing_if = "Option::is_none" });
            }

            if self.args.skip_deserializing || self.serde_args.skip_deserializing {
                meta.push(quote! { skip_deserializing });
            }
        }

        quote! {
            #(#meta),*
        }
    }

    /// Return the type to use within the partial, which is always
    /// wrapped in an `Option`, unless the type is already optional.
    pub fn get_partial_type(&self) -> TokenStream {
        let ty = self.value.get_partial_type();

        if self.value.is_outer_option_wrapped() {
            quote! { #ty }
        } else {
            quote! { Option<#ty> }
        }
    }
}

// Only used for partials!
impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = self.get_partial_attributes();
        let vis = &self.vis;
        let ty = self.get_partial_type();

        if let Some(name) = &self.ident {
            tokens.extend(quote! {
                #(#attrs)*
                #vis #name: #ty,
            });
        } else {
            tokens.extend(quote! {
                #(#attrs)*
                #vis #ty,
            });
        }
    }
}

#[cfg(feature = "schema")]
impl Field {
    /// Generate the schema for this setting. When `as_field` is true, it's
    /// wrapped in a `SchemaField` keyed by name, otherwise the bare schema
    /// is returned, for use within tuples.
    pub fn impl_schema_type(&self, as_field: bool) -> TokenStream {
        use crate::utils::{extract_comment, extract_deprecated};
        use syn::Lit;

        let ty = &self.value.ty;

        // Nested configs are inferred as partials, so that the
        // schema can be marked as such
        let mut schema = if self.is_nested() {
            quote! { schema.infer_as_nested::<#ty>() }
        } else {
            quote! { schema.infer::<#ty>() }
        };

        // Literal defaults are rendered within the schema
        if let Some(Expr::Lit(lit)) = &self.args.default {
            let value = match &lit.lit {
                Lit::Str(v) => Some(quote! { LiteralValue::String(#v.into()) }),
                Lit::Int(v) => Some(if v.suffix().starts_with('u') {
                    quote! { LiteralValue::Uint(#v) }
                } else {
                    quote! { LiteralValue::Int(#v) }
                }),
                Lit::Float(v) => Some(if v.suffix() == "f32" {
                    quote! { LiteralValue::F32(#v) }
                } else {
                    quote! { LiteralValue::F64(#v) }
                }),
                Lit::Bool(v) => Some(quote! { LiteralValue::Bool(#v) }),
                _ => None,
            };

            if let Some(value) = value {
                schema = quote! { schema.infer_with_default::<#ty>(#value) };
            }
        }

        let comment = extract_comment(&self.attrs);
        let deprecated = extract_deprecated(&self.attrs);

        // Tuple items only support a description
        if !as_field {
            return match comment {
                Some(comment) => quote! {
                    {
                        let mut schema = #schema;
                        schema.set_description(#comment);
                        schema
                    }
                },
                None => schema,
            };
        }

        let mut statements = vec![];

        let aliases = self.get_aliases();

        if !aliases.is_empty() {
            statements.push(quote! {
                field.aliases = [#(#aliases),*]
                    .into_iter()
                    .map(|alias| alias.to_string())
                    .collect::<Vec<_>>();
            });
        }

        if let Some(comment) = comment {
            statements.push(quote! { field.comment = Some(#comment.into()); });
        }

        if let Some(deprecated) = deprecated {
            statements.push(quote! { field.deprecated = Some(#deprecated.into()); });
        }

        // Only explicit keys are known statically, as derived
        // keys depend on the prefix in effect at runtime
        if let Some(EnvKey::Explicit(env_var)) = self.get_env_var() {
            statements.push(quote! { field.env_var = Some(#env_var.into()); });
        }

        for (name, enabled) in [
            ("flatten", self.is_flatten()),
            ("hidden", self.is_skipped()),
            ("nullable", self.is_nullable()),
            ("optional", self.is_optional()),
        ] {
            if enabled {
                let name = format_ident!("{name}");

                statements.push(quote! { field.#name = true; });
            }
        }

        let name = self.get_name_or_index();

        if statements.is_empty() {
            quote! { (#name.into(), SchemaField::new(#schema)) }
        } else {
            quote! {
                (#name.into(), {
                    let mut field = SchemaField::new(#schema);
                    #(#statements)*
                    field
                })
            }
        }
    }
}

impl Field {
    pub fn impl_full_from_partial(&self) -> ImplResult {
        let key = self.get_key();

        // Reset extendable values since we don't have the entire resolved list
        if self.is_extendable() {
            return ImplResult {
                value: quote! { Default::default() },
                ..Default::default()
            };
        }

        // Nested partials must be converted, and stripped wrappers reapplied
        let value = if self.value.requires_from_partial_mapping() {
            let data_var = format_ident!("data");
            let inner = self.value.impl_full_from_partial_value(&data_var).value;

            // When option wrapped, the partial's `Option` is the first layer,
            // so pass the value as-is and let the layer unwrap it
            let data = if self.value.is_outer_option_wrapped() {
                quote! { partial.#key }
            } else {
                quote! { partial.#key.unwrap_or_default() }
            };

            quote! {
                {
                    let #data_var = #data;
                    #inner
                }
            }
        } else if self.value.is_outer_option_wrapped() {
            // Use optional values as-is as they're already wrapped in `Option`
            quote! { partial.#key }
        } else {
            // Otherwise unwrap the resolved value or use the type default
            quote! { partial.#key.unwrap_or_default() }
        };

        ImplResult {
            value,
            ..Default::default()
        }
    }

    pub fn impl_partial_default_value(&self) -> ImplResult {
        self.value.impl_partial_default_value(&self.args)
    }

    pub fn impl_partial_env_value(&self) -> ImplResult {
        if self.is_nested() {
            return self.value.impl_partial_env_value(&self.args, None);
        }

        match self.get_env_var() {
            Some(env_key) => self
                .value
                .impl_partial_env_value(&self.args, Some(&env_key)),
            None => ImplResult::skipped(),
        }
    }

    pub fn impl_partial_extends_from(&self) -> ImplResult {
        if self.is_extendable() {
            self.value
                .impl_partial_extends_from(&self.args, &self.get_key())
        } else {
            ImplResult::skipped()
        }
    }

    pub fn impl_partial_finalize(&self) -> ImplResult {
        if !self.is_nested() && self.args.transform.is_none() {
            return ImplResult::skipped();
        }

        let key = self.get_key();

        let mut value = if self.is_nested() {
            // The `if let` below consumes the partial's `Option`, which is
            // the first `Option` layer when the type is optional
            self.value
                .impl_partial_finalize_nested(&format_ident!("layer"), true)
                .value
        } else {
            quote! { layer }
        };

        if let Some(func) = &self.args.transform {
            value = quote! { #func(#value, context)? };
        };

        ImplResult {
            value: quote! {
                if let Some(layer) = partial.#key {
                    partial.#key = Some(#value);
                }
            },
            ..Default::default()
        }
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        self.value.impl_partial_merge(&self.args, &self.get_key())
    }

    pub fn impl_partial_validate(&self) -> ImplResult {
        let key = self.get_key();
        let key_string = self.get_name_or_index();
        let res = self.value.impl_partial_validate(&self.args, &key_string);
        let mut inner = res.value;
        let mut has_inner = !res.no_value;

        if self.is_nested() {
            let setting_var = format_ident!("setting");
            // The `if let` below consumes the partial's `Option`
            let nested_value = self
                .value
                .impl_partial_validate_nested(&quote! { #key_string }, &setting_var, false, true)
                .value;

            has_inner = true;
            inner = quote! {
                #inner
                #nested_value
            };
        }

        let mut has_outer = has_inner;
        let mut outer = if has_inner {
            quote! {
                if let Some(setting) = &self.#key {
                    #inner
                }
            }
        } else {
            quote! {}
        };

        if self.is_required() {
            has_outer = true;
            outer = quote! {
                #outer

                if self.#key.is_none() {
                    validate.required(#key_string);
                }
            };
        }

        if has_outer {
            ImplResult {
                value: outer,
                ..Default::default()
            }
        } else {
            ImplResult::skipped()
        }
    }
}

struct Index(usize);

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::usize_unsuffixed(self.0));
    }
}
