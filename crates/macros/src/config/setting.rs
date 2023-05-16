use crate::config::setting_type::SettingType;
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Expr, ExprLit, ExprPath, Field, Lit, Meta, Type};

// #[setting()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct SettingArgs {
    pub default: Option<Expr>,
    pub default_fn: Option<ExprPath>,
    pub default_str: Option<String>,
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

impl SettingArgs {
    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        if let Some(rename) = &self.rename {
            meta.push(quote! { rename = #rename });
        }

        if self.skip {
            meta.push(quote! { skip });
        } else {
            meta.push(quote! { skip_serializing_if = "Option::is_none" });
        }

        quote! {
            #(#meta),*
        }
    }
}

pub struct Setting<'l> {
    pub args: SettingArgs,
    pub comment: Option<String>,
    pub name: &'l Ident,
    pub value: &'l Type,
    pub value_type: SettingType<'l>,
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        let args = SettingArgs::from_attributes(&field.attrs).unwrap_or_default();

        if args.default_fn.is_some() && args.default.is_some() {
            panic!("Cannot provide both `default` and `default_fn`.");
        }

        if args.default_str.is_some() && args.default.is_some() {
            panic!("Cannot provide both `default` and `default_str`.");
        }

        if args.parse_env.is_some() && args.env.is_none() {
            panic!("Cannot use `parse_env` without `env`.");
        }

        if args.validate.is_some() && args.nested {
            panic!("Cannot use `validate` for `nested` configs.");
        }

        let setting = Setting {
            comment: extract_comment(field),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
            value_type: if args.nested {
                SettingType::nested(&field.ty)
            } else {
                SettingType::value(&field.ty)
            },
            args,
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
            || self.args.default_fn.is_some()
            || self.args.default_str.is_some()
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
        let value = self.value_type.get_default_value(self.name, &self.args);

        match (&self.args.env, &self.args.parse_env) {
            (Some(env), Some(parse_env)) => {
                quote! {
                    schematic::internal::parse_from_env_var(#env, #parse_env, #value)?
                }
            }
            (Some(env), None) => {
                quote! {
                    schematic::internal::default_from_env_var(#env, #value)?
                }
            }
            _ => value,
        }
    }

    pub fn get_from_statement(&self) -> TokenStream {
        let name = self.name;

        // let struct_name = self.get_nested_struct_name();

        // if self.is_optional() {
        //     quote! { partial.#name.map(#struct_name::from_partial) }
        // } else {
        //     quote! { #struct_name::from_partial(partial.#name.unwrap_or_default()) }
        // }

        self.value_type.get_from_value(name, &self.args)
    }

    pub fn get_merge_statement(&self) -> TokenStream {
        let name = self.name;

        if let Some(func) = self.args.merge.as_ref() {
            quote! {
                if self.#name.is_some() && next.#name.is_some() {
                    self.#name = #func(self.#name.take().unwrap(), next.#name.take().unwrap());
                } else if next.#name.is_some() {
                    self.#name = next.#name;
                }
            }
        } else {
            quote! {
                if next.#name.is_some() {
                    self.#name = next.#name;
                }
            }
        }
    }

    pub fn get_validate_statement(&self) -> TokenStream {
        let name = self.name;
        let validator = self
            .value_type
            .get_validate_statement(self.name, &self.args);

        if self.is_optional() {
            quote! {
                if let Some(setting) = self.#name.as_ref() {
                    #validator
                }
            }
        } else {
            quote! {
                let setting = &self.#name;
                #validator
            }
        }
    }
}

impl<'l> ToTokens for Setting<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = self.value;

        // Wrap value based on current state
        let value = match (self.is_nested(), self.is_optional()) {
            (true, true) => {
                // let inner_value = self.inner_value.unwrap();
                // let value_cast = quote! { <#inner_value as schematic::Config>::Partial };

                // if let Some(vec_value) = unwrap_vec(inner_value) {
                //     quote! { Option<Vec<#value_cast>> }
                // } else {
                //     quote! { Option<#value_cast> }
                // }
                quote! { None }
            }
            (true, false) => {
                quote! { Option<<#value as schematic::Config>::Partial> }
            }
            (false, true) => {
                quote! { #value }
            }
            (false, false) => {
                quote! { Option<#value> }
            }
        };

        // Gather all attributes
        let serde_meta = self.args.get_serde_meta();
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

fn extract_comment(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("doc") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &meta.value
                {
                    return Some(value.value());
                }
            }
        }
    }

    None
}
