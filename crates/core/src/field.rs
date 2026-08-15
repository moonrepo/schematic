use crate::args::{
    NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeIoDirection, SerdeRenameArg,
};
use crate::container::ContainerArgs;
use crate::field_value::FieldValue;
use crate::utils::{ImplResult, is_inheritable_attribute, preserve_str_literal};
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
    #[cfg(feature = "env")]
    pub env: Option<String>,
    #[cfg(feature = "env")]
    pub env_prefix: Option<String>,
    #[cfg(feature = "schema")]
    pub exclude: bool,
    #[cfg(feature = "extends")]
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<NestedArg>,
    #[cfg(feature = "env")]
    pub parse_env: Option<ExprPath>,
    pub partial: Option<PartialArg>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    #[cfg(feature = "validate")]
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
    pub fn get_env_var(&self) -> Option<String> {
        None
    }

    #[cfg(feature = "env")]
    pub fn get_env_var(&self) -> Option<String> {
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

            return Some(env_key.to_owned());
        }

        // When the container has a prefix, we use the field name as a key
        if self.container_args.env_prefix.is_some() {
            return Some(self.get_name().to_uppercase());
        }

        if self.args.parse_env.is_some() {
            panic!("Cannot use `parse_env` without `env` or a parent `env_prefix`.");
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

        self.get_name_original().to_string()
    }

    pub fn get_name_original(&self) -> &Ident {
        self.ident
            .as_ref()
            .expect("Name only usable on named fields!")
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

    pub fn is_nested(&self) -> bool {
        self.args
            .nested
            .as_ref()
            .is_some_and(|nested| nested.is_nested())
    }

    pub fn is_required(&self) -> bool {
        self.args.required
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

        let value = if self.is_nested() {
            let data_var = format_ident!("data");
            let inner = self.value.impl_full_from_partial_nested(&data_var).value;

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
            return self.value.impl_partial_env_value(&self.args, "");
        }

        match self.get_env_var() {
            Some(env_key) => self.value.impl_partial_env_value(&self.args, &env_key),
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
        let key_string = key.to_string();
        let res = self.value.impl_partial_validate(&self.args, &key);
        let mut inner = res.value;
        let mut has_inner = !res.no_value;

        if self.is_nested() {
            let setting_var = format_ident!("setting");
            // The `if let` below consumes the partial's `Option`
            let nested_value = self
                .value
                .impl_partial_validate_nested(&key_string, &setting_var, true)
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
