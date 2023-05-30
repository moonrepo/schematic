use crate::config::setting_type::SettingType;
use crate::utils::{extract_comment, preserve_str_literal};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Expr, ExprPath, Field, Type};

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeArgs {
    pub rename: Option<String>,
    pub skip: bool,
}

// #[setting()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct SettingArgs {
    #[darling(with = "preserve_str_literal", map = "Some")]
    pub default: Option<Expr>,
    pub env: Option<String>,
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    pub parse_env: Option<ExprPath>,
    pub validate: Option<Expr>,

    // serde
    pub rename: Option<String>,
    pub skip: bool,
}

pub struct Setting<'l> {
    pub args: SettingArgs,
    pub serde_args: SerdeArgs,
    pub comment: Option<String>,
    pub name: &'l Ident,
    pub value: &'l Type,
    pub value_type: SettingType<'l>,
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        let args = SettingArgs::from_attributes(&field.attrs).unwrap_or_default();
        let serde_args = SerdeArgs::from_attributes(&field.attrs).unwrap_or_default();

        if args.parse_env.is_some() && args.env.is_none() {
            panic!("Cannot use `parse_env` without `env`.");
        }

        if args.validate.is_some() && args.nested {
            panic!("Cannot use `validate` for `nested` configs.");
        }

        let setting = Setting {
            comment: extract_comment(&field.attrs),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
            value_type: if args.nested {
                SettingType::nested(&field.ty)
            } else {
                SettingType::value(&field.ty)
            },
            args,
            serde_args,
        };

        if setting.has_default() {
            if setting.is_nested() {
                panic!("Cannot use defaults with `nested` configs.");
            }

            if setting.is_optional() {
                panic!("Cannot use defaults with optional settings.");
            }
        }

        setting
    }

    pub fn has_default(&self) -> bool {
        self.args.default.is_some()
    }

    pub fn is_extendable(&self) -> bool {
        self.args.extend
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn is_optional(&self) -> bool {
        self.value_type.is_optional()
    }

    pub fn get_default_statement(&self) -> TokenStream {
        if let Some(value) = self.value_type.get_default_value(self.name, &self.args) {
            let name = self.name;

            return quote! { partial.#name = #value; };
        }

        quote! {}
    }

    pub fn get_env_statement(&self) -> TokenStream {
        if let Some(value) = self.value_type.get_env_value(&self.args) {
            let name = self.name;

            return quote! { partial.#name = #value; };
        }

        quote! {}
    }

    pub fn get_from_statement(&self) -> TokenStream {
        self.value_type.get_from_value(self.name, &self.args)
    }

    pub fn get_merge_statement(&self) -> TokenStream {
        self.value_type.get_merge_statement(self.name, &self.args)
    }

    pub fn get_validate_statement(&self) -> TokenStream {
        let name = self.name;

        let Some(validator) = self
            .value_type
            .get_validate_statement(self.name, &self.args) else {
            return quote! {};
        };

        quote! {
            if let Some(setting) = self.#name.as_ref() {
                #validator
            }
        }
    }

    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        if let Some(rename) = &self.args.rename {
            meta.push(quote! { rename = #rename });
        } else if let Some(rename) = &self.serde_args.rename {
            meta.push(quote! { rename = #rename });
        }

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
        } else {
            meta.push(quote! { skip_serializing_if = "Option::is_none" });
        }

        quote! {
            #(#meta),*
        }
    }
}

impl<'l> ToTokens for Setting<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = &self.value_type;

        // Gather all attributes
        let serde_meta = self.get_serde_meta();
        let mut attrs = vec![quote! { #[serde(#serde_meta)] }];

        if let Some(cmt) = &self.comment {
            attrs.push(quote! { #[doc = #cmt] });
        };

        tokens.extend(quote! {
             #(#attrs)*
            pub #name: #value,
        });
    }
}
