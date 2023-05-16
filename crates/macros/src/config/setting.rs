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
    default_str: Option<String>,
    env: Option<String>,
    extend: bool,
    merge: Option<ExprPath>,
    nested: bool,
    parse_env: Option<ExprPath>,
    validate: Option<Expr>,

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
    pub inner_value: Option<&'l Type>,

    optional: bool,
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

        let inner_value = unwrap_option(&field.ty);
        let setting = Setting {
            args,
            comment: extract_comment(field),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
            inner_value,
            optional: inner_value.is_some(),
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
        self.optional
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

            if self.is_optional() {
                return quote! { None };
            }

            return quote! { Some(#struct_name::default_values()?) };
        };

        if let Some(func) = self.args.default_fn.as_ref() {
            return quote! { Some(#func()) };
        };

        if let Some(string) = self.args.default_str.as_ref() {
            let value = self.value;

            return quote! {
                Some(
                    #value::try_from(#string)
                        .map_err(|e| schematic::ConfigError::InvalidDefault(e.to_string()))?
                )
            };
        };

        let Some(expr) = self.args.default.as_ref() else {
            return quote! { None };
        };

        match expr {
            Expr::Array(_) | Expr::Call(_) | Expr::Lit(_) | Expr::Tuple(_) => {
                quote! { Some(#expr) }
            }
            invalid => {
                let name = self.name.to_string();
                let info = format!("{:?}", invalid);

                panic!("Unsupported default value for {name} ({info}). May only provide literals, arrays, or tuples. Use `default_fn` or `default_str` for more complex defaults.");
            }
        }
    }

    pub fn get_from_statement(&self) -> TokenStream {
        let name = self.name;

        // Recursively build nested from partial
        if self.is_nested() {
            let struct_name = self.get_nested_struct_name();

            if self.is_optional() {
                quote! { partial.#name.map(#struct_name::from_partial) }
            } else {
                quote! { #struct_name::from_partial(partial.#name.unwrap_or_default()) }
            }

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

    pub fn get_validate_statement(&self) -> TokenStream {
        let name = self.name;
        let name_quoted = format!("{}", self.name);

        let validator = if self.is_nested() {
            quote! {
                if let Err(nested_error) = setting.validate_with_path(path.join_key(#name_quoted)) {
                    errors.push(schematic::ValidateErrorType::nested(nested_error));
                }
            }
        } else if let Some(expr) = self.args.validate.as_ref() {
            let func = match expr {
                Expr::Call(func) => quote! { #func },
                Expr::Path(func) => quote! { #func },
                _ => {
                    panic!("Unsupported `validate` syntax.");
                }
            };

            quote! {
                if let Err(error) = #func(setting) {
                    errors.push(schematic::ValidateErrorType::setting(
                        path.join_key(#name_quoted),
                        error,
                    ));
                }
            }
        } else {
            return quote! {};
        };

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

    pub fn get_nested_struct_name(&self) -> Ident {
        let inner = self.inner_value.unwrap_or(self.value);

        match inner {
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
        let value = match (self.is_nested(), self.is_optional()) {
            (true, true) => {
                let inner_value = self.inner_value;

                quote! { Option<<#inner_value as schematic::Config>::Partial> }
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
