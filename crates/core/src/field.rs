use crate::args::{
    NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeIoDirection, SerdeRenameArg,
};
use crate::container::ContainerArgs;
use crate::field_value::FieldValue;
use crate::utils::{ImplResult, preserve_str_literal};
use darling::FromAttributes;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, Expr, ExprPath, Field as NativeField, FieldMutability, Ident, Visibility};

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
    pub mutability: FieldMutability,
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
            mutability: field.mutability,
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

// impl ToTokens for Field {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let mut value = self.value.ty_string.clone();

//         if let Some(nested_ident) = &self.value.nested_ident {
//             let ident = nested_ident.to_string();

//             value = value.replace(&ident, &format!("<{ident} as schematic::Config>::Partial"));
//         }

//         if !self.value.is_outer_option_wrapped() {
//             value = format!("Option<{value}>");
//         }

//         let key = self.ident.as_ref().unwrap();
//         let value: TokenStream = parse_str(&value).unwrap();

//         tokens.extend(quote! {
//             pub #key: #value,
//         });
//     }
// }

impl Field {
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
            self.value
                .impl_partial_finalize_nested(&format_ident!("layer"))
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
            let nested_value = self
                .value
                .impl_partial_validate_nested(&key_string, &setting_var)
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
                    validate.required(#key);
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
