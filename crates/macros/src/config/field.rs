use crate::common::{Field, FieldValue};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Expr;

impl<'l> Field<'l> {
    pub fn generate_default_value(&self) -> TokenStream {
        if self.is_optional() {
            quote! { None }
        } else {
            self.value_type.generate_default_value(&self.args)
        }
    }

    pub fn generate_env_statement(&self) -> Option<TokenStream> {
        if self.is_nested() {
            return None;
        }

        let env = self.get_env_var();

        if env.is_none() {
            if self.args.parse_env.is_some() {
                panic!("Cannot use `parse_env` without `env` or a parent `env_prefix`.");
            }

            return None;
        };

        let value = if let Some(parse_env) = &self.args.parse_env {
            quote! {
                schematic::internal::parse_from_env_var(#env, #parse_env)?
            }
        } else {
            quote! {
                schematic::internal::default_from_env_var(#env)?
            }
        };

        let key = self.get_field_key();

        Some(quote! { partial.#key = #value; })
    }

    pub fn generate_finalize_statement(&self) -> TokenStream {
        if let Some(value) = self.value_type.get_finalize_value() {
            let key = self.get_field_key();

            return quote! {
                if let Some(data) = partial.#key {
                    partial.#key = Some(#value);
                }
            };
        }

        quote! {}
    }

    pub fn generate_from_partial_value(&self) -> TokenStream {
        let key = self.get_field_key();

        #[allow(clippy::collapsible_else_if)]
        if matches!(self.value_type, FieldValue::Value { .. }) {
            let mut value = if self.args.extend {
                // Reset extendable values since we don't have the entire resolved list
                quote! { Default::default() }
            } else if self.is_optional() {
                // Use optional values as-is as they're already wrapped in `Option`
                quote! { partial.#key }
            } else {
                // Otherwise unwrap the resolved value or use the type default
                quote! { partial.#key.unwrap_or_default() }
            };

            if self.value_type.is_outer_boxed() {
                value = quote! { Box::new(#value) };
            }

            value
        } else {
            let mut value = self.value_type.get_from_partial_value();

            if self.value_type.is_outer_boxed() {
                value = quote! { Box::new(#value) };
            }

            if self.is_optional() {
                quote! {
                    if let Some(data) = partial.#key {
                        Some(#value)
                    } else {
                        None
                    }
                }
            } else {
                quote! {
                    {
                        let data = partial.#key.unwrap_or_default();
                        #value
                    }
                }
            }
        }
    }

    pub fn generate_merge_statement(&self) -> TokenStream {
        let key = self.get_field_key();

        self.value_type.get_merge_statement(&key, &self.args)
    }

    pub fn generate_validate_statement(&self) -> TokenStream {
        let key = self.get_field_key();
        let key_quoted = format!("{key}");
        let mut stmts = vec![];

        if let Some(expr) = self.args.validate.as_ref() {
            let func = match expr {
                // func(arg)()
                Expr::Call(func) => quote! { #func },
                // func()
                Expr::Path(func) => quote! { #func },
                _ => {
                    panic!("Unsupported `validate` syntax.");
                }
            };

            stmts.push(quote! {
                if let Err(error) = #func(setting, self, context, finalize) {
                    errors.push(schematic::ValidateErrorType::setting(
                        path.join_key(#key_quoted),
                        error,
                    ));
                }
            });
        }

        if let Some(validator) = self.value_type.get_validate_statement(self.get_name_raw()) {
            stmts.push(validator);
        }

        let first = if stmts.is_empty() {
            quote! {}
        } else {
            quote! {
                if let Some(setting) = self.#key.as_ref() {
                    #(#stmts)*
                }
            }
        };

        let second = if self.is_required() {
            quote! {
                if finalize && self.#key.is_none() {
                    errors.push(schematic::ValidateErrorType::setting_required(
                        path.join_key(#key_quoted),
                    ));
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #first
            #second
        }
    }

    fn get_field_key(&self) -> Ident {
        self.name
            .as_ref()
            .map(|name| format_ident!("{name}"))
            .unwrap_or_else(|| format_ident!("{}", self.index.to_string()))
    }
}
