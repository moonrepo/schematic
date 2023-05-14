use crate::utils::unwrap_option;
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprLit, ExprPath, Field, Lit, Meta, Type};

// #[setting()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct SettingArgs {
    default: Option<Expr>,
    default_fn: Option<ExprPath>,
    env: Option<String>,
    extend: bool,
    merge: Option<ExprPath>,
    nested: bool,
    parse_env: Option<ExprPath>,
    validate: Option<ExprPath>,

    // serde
    rename: Option<String>,
    skip: bool,
}

impl SettingArgs {
    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        if let Some(rename) = &self.rename {
            meta.push(quote! { rename = #rename });
        }

        if self.skip {
            meta.push(quote! { skip });
        }

        meta.push(quote! { skip_serializing_if = "Option::is_none" });

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
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        let args = SettingArgs::from_attributes(&field.attrs).unwrap_or_default();

        if args.default_fn.is_some() && args.default.is_some() {
            panic!("Cannot provide both `default` and `default_fn`.");
        }

        if args.parse_env.is_some() && args.env.is_none() {
            panic!("Cannot use `parse_env` without `env`.");
        }

        let setting = Setting {
            args,
            comment: extract_comment(field),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
        };

        if setting.has_default() {
            if setting.is_nested() {
                panic!("Cannot use `default` or `default_fn` with nested configs.");
            }

            if setting.is_optional() {
                panic!("Cannot use `default` or `default_fn` with optional settings.");
            }
        }

        setting
    }

    pub fn has_default(&self) -> bool {
        self.args.default.is_some() || self.args.default_fn.is_some()
    }

    pub fn has_validation(&self) -> bool {
        self.args.validate.is_some()
    }

    pub fn is_extendable(&self) -> bool {
        self.args.extend
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn is_optional(&self) -> bool {
        unwrap_option(self.value).is_some()
    }

    pub fn get_default_statement(&self) -> TokenStream {
        let value = self.get_default_value();

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

    pub fn get_default_value(&self) -> TokenStream {
        if self.args.nested {
            let struct_name = format_ident!("Partial{}", self.get_nested_struct_name());

            return quote! { Some(#struct_name::default_values()?) };
        };

        if let Some(func) = self.args.default_fn.as_ref() {
            return quote! { Some(#func()) };
        };

        let Some(expr) = self.args.default.as_ref() else {
            return quote! { None };
        };

        match expr {
            Expr::Array(_) | Expr::Call(_) | Expr::Lit(_) | Expr::Tuple(_) => {
                quote! { Some(#expr) }
            }
            // Strings are `Path` for some reason instead of `Lit`...
            Expr::Path(inner) => {
                let string = format!("{}", inner.to_token_stream());
                quote! { Some(#string.into()) }
            }
            invalid => {
                let name = self.name.to_string();
                let info = format!("{:?}", invalid);

                panic!("Unsupported default value for {name} ({info}). May only provide literals, arrays, or tuples. Use `default_fn` for more complex defaults.");
            }
        }
    }

    pub fn get_from_statement(&self) -> TokenStream {
        let name = self.name;

        // Recursively build nested from partial
        if self.is_nested() {
            let struct_name = self.get_nested_struct_name();

            quote! { #struct_name::from_partial(partial.#name.unwrap_or_default()) }

            // Reset extendable values since we don't have the entire resolved list
        } else if self.is_extendable() {
            quote! { Default::default() }

            // Use optional values as-is as they're already wrapped in `Option`
        } else if self.is_optional() {
            quote! { partial.#name }

            // Otherwise unwrap the resolved value or use the type default
        } else {
            quote! { partial.#name.unwrap_or_default() }
        }
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

    pub fn get_nested_struct_name(&self) -> Ident {
        match &self.value {
            Type::Path(path) => {
                let segments = &path.path.segments;
                let last_segment = segments.last().unwrap();

                last_segment.ident.clone()
            }
            _ => panic!("Only structs are supported for nested settings."),
        }
    }
}

impl<'l> ToTokens for Setting<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = self.value;

        // Wrap value based on current state
        let mut value = if self.is_nested() {
            quote! { <#value as schematic::Config>::Partial }
        } else {
            quote! { #value }
        };

        if !self.is_optional() {
            value = quote! { Option<#value> };
        }

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
