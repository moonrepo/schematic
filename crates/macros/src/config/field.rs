use crate::common::{Field, FieldValue};
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, TokenStreamExt, quote};

impl Field<'_> {
    pub fn generate_default_value(&self) -> TokenStream {
        self.value_type
            .generate_default_value(&self.args, self.is_nullable(), self.is_required())
    }

    #[cfg(not(feature = "env"))]
    pub fn generate_env_statement(&self) -> Option<TokenStream> {
        None
    }

    #[cfg(feature = "env")]
    pub fn generate_env_statement(&self) -> Option<TokenStream> {
        let key = self.get_field_key();

        if self.is_nested() {
            return self
                .value_type
                .generate_env_value(&self.args, "")
                .map(|value| quote! { partial.#key = #value; });
        }

        let Some(env_key) = self.get_env_var() else {
            if self.args.parse_env.is_some() {
                panic!("Cannot use `parse_env` without `env` or a parent `env_prefix`.");
            }

            return None;
        };

        self.value_type
            .generate_env_value(&self.args, &env_key)
            .map(|value| quote! { partial.#key = #value; })
    }

    pub fn generate_finalize_statement(&self) -> TokenStream {
        let key = self.get_field_key();

        match (self.value_type.get_finalize_value(), &self.args.transform) {
            (Some(value), Some(func)) => {
                quote! {
                    if let Some(data) = partial.#key {
                        partial.#key = Some(#func(#value, context)?);
                    }
                }
            }
            (Some(value), None) => {
                quote! {
                    if let Some(data) = partial.#key {
                        partial.#key = Some(#value);
                    }
                }
            }
            (None, Some(func)) => {
                quote! {
                    if let Some(data) = partial.#key {
                        partial.#key = Some(#func(data, context)?);
                    }
                }
            }
            _ => quote! {},
        }
    }

    pub fn generate_from_partial_value(&self) -> TokenStream {
        let key = self.get_field_key();
        let key_quoted = self.get_field_key_string();

        #[allow(clippy::collapsible_else_if)]
        if matches!(self.value_type, FieldValue::Value { .. }) {
            // Reset extendable values since we don't have the entire resolved list
            #[cfg(feature = "extends")]
            if self.args.extend {
                return quote! { Default::default() };
            }

            if self.value_type.is_outer_boxed() {
                let mut value = quote! { Box::new(partial.#key.unwrap_or_default()) };

                if self.is_nullable() {
                    value = quote! { Some(#value) };
                }

                value
            } else {
                if self.is_nullable() {
                    // Use optional values as-is as they're already wrapped in `Option`
                    quote! { partial.#key }
                } else if self.is_required() {
                    // Trigger a validation error if the value is missing
                    quote! { partial.#key.ok_or(schematic::ConfigError::MissingRequired(#key_quoted.into()))? }
                } else {
                    // Otherwise unwrap the resolved value or use the type default
                    quote! { partial.#key.unwrap_or_default() }
                }
            }
        } else {
            let mut value = self.value_type.get_from_partial_value();

            if self.value_type.is_outer_boxed() {
                value = quote! { Box::new(#value) };
            }

            if self.is_nullable() {
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
        self.value_type
            .get_merge_statement(self.get_field_key(), &self.args)
    }

    pub fn generate_validate_statement(&self) -> TokenStream {
        let key = self.get_field_key();
        let key_quoted = self.get_field_key_string();
        let mut stmts = vec![];

        #[cfg(feature = "validate")]
        if let Some(expr) = self.args.validate.as_ref() {
            use syn::Expr;

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
                if let Err(mut error) = #func(setting, self, context, finalize) {
                    errors.push(error.prepend_path(path.join_key(#key_quoted)));
                }
            });
        }

        if let Some(validator) = self.value_type.get_validate_statement(&key_quoted) {
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
                    errors.push(schematic::ValidateError::required().prepend_path(
                        path.join_key(#key_quoted)
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

    fn get_field_key(&self) -> TokenStream {
        self.name
            .as_ref()
            .map(|name| quote! { #name })
            .unwrap_or_else(|| {
                let index = Index(self.index);

                quote! { #index }
            })
    }

    fn get_field_key_string(&self) -> String {
        self.name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| self.index.to_string())
    }
}

struct Index(usize);

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Literal::usize_unsuffixed(self.0));
    }
}
