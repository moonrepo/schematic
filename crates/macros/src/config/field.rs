use crate::common::{Field, FieldValue};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

impl<'l> Field<'l> {
    pub fn generate_default_value(&self) -> TokenStream {
        if self.is_optional() {
            quote! { None }
        } else {
            self.value_type
                .generate_default_value(self.name, &self.args)
        }
    }

    pub fn generate_env_statement(&self) -> Option<TokenStream> {
        if self.is_nested() {
            return None;
        }

        let name = self.name;
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

        Some(quote! { partial.#name = #value; })
    }

    pub fn generate_finalize_statement(&self) -> TokenStream {
        if let Some(value) = self.value_type.get_finalize_value() {
            let name = self.name;

            return quote! {
                if let Some(data) = partial.#name {
                    partial.#name = Some(#value);
                }
            };
        }

        quote! {}
    }

    pub fn generate_from_partial_value(&self) -> TokenStream {
        let name = self.name;
        let value = self.value_type.get_from_partial_value();

        #[allow(clippy::collapsible_else_if)]
        if matches!(self.value_type, FieldValue::Value { .. }) {
            // Reset extendable values since we don't have the entire resolved list
            if self.args.extend {
                quote! { Default::default() }

                // Use optional values as-is as they're already wrapped in `Option`
            } else if self.is_optional() {
                quote! { partial.#name }

                // Otherwise unwrap the resolved value or use the type default
            } else {
                quote! { partial.#name.unwrap_or_default() }
            }
        } else {
            if self.is_optional() {
                quote! {
                    if let Some(data) = partial.#name {
                        Some(#value)
                    } else {
                        None
                    }
                }
            } else {
                quote! {
                    {
                        let data = partial.#name.unwrap_or_default();
                        #value
                    }
                }
            }
        }
    }

    pub fn generate_merge_statement(&self) -> TokenStream {
        self.value_type.get_merge_statement(self.name, &self.args)
    }

    pub fn generate_validate_statement(&self) -> TokenStream {
        let name = self.name;
        let name_quoted = format!("{name}");
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
                if let Err(error) = #func(setting, self, context) {
                    errors.push(schematic::ValidateErrorType::setting(
                        path.join_key(#name_quoted),
                        error,
                    ));
                }
            });
        }

        if let Some(validator) = self.value_type.get_validate_statement(self.name) {
            stmts.push(validator);
        }

        if stmts.is_empty() {
            quote! {}
        } else if self.is_required() {
            quote! {
                if let Some(setting) = self.#name.as_ref() {
                    #(#stmts)*
                } else if finalize {
                    errors.push(schematic::ValidateErrorType::setting_required(
                        path.join_key(#name_quoted),
                    ));
                }
            }
        } else {
            quote! {
                if let Some(setting) = self.#name.as_ref() {
                    #(#stmts)*
                }
            }
        }
    }
}
