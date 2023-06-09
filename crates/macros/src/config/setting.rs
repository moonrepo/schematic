use crate::config::setting_type::SettingType;
use crate::utils::{
    extract_comment, extract_common_attrs, format_case, has_attr, preserve_str_literal,
};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Field, Lit, Type};

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
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l Type,
    pub value_type: SettingType<'l>,
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        let args = SettingArgs::from_attributes(&field.attrs).unwrap_or_default();
        let serde_args = SerdeArgs::from_attributes(&field.attrs).unwrap_or_default();

        let setting = Setting {
            name: field.ident.as_ref().unwrap(),
            attrs: extract_common_attrs(&field.attrs),
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

    pub fn is_skipped(&self) -> bool {
        self.args.skip || self.serde_args.skip
    }

    pub fn get_name(&self, casing_format: Option<&str>) -> String {
        if let Some(local) = &self.args.rename {
            local.to_owned()
        } else if let Some(serde) = &self.serde_args.rename {
            serde.to_owned()
        } else if let Some(format) = casing_format {
            format_case(format, &self.name.to_string(), false)
        } else {
            self.name.to_string()
        }
    }

    pub fn generate_default_value(&self) -> TokenStream {
        if self.is_optional() {
            quote! { None }
        } else {
            self.value_type
                .generate_default_value(self.name, &self.args)
        }
    }

    pub fn generate_env_statement(&self, prefix: Option<&String>) -> Option<TokenStream> {
        if self.is_nested() {
            return None;
        }

        let name = self.name;

        let env = if let Some(env_name) = &self.args.env {
            env_name.to_owned()
        } else if let Some(env_prefix) = prefix {
            format!("{}{}", env_prefix, self.get_name(None)).to_uppercase()
        } else {
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
        if matches!(self.value_type, SettingType::Value { .. }) {
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
        let mut stmts = vec![];

        if let Some(expr) = self.args.validate.as_ref() {
            let name_quoted = format!("{}", name);

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
        } else {
            quote! {
                if let Some(setting) = self.#name.as_ref() {
                    #(#stmts)*
                }
            }
        }
    }

    pub fn generate_schema_type(&self, casing_format: &str) -> TokenStream {
        let name = self.get_name(Some(casing_format));
        let value = self.value;

        let deprecated = has_attr(&self.attrs, "deprecated");
        let hidden = self.is_skipped();
        let nullable = self.is_optional();
        let partial = self.is_nested();

        let description = if let Some(comment) = extract_comment(&self.attrs) {
            quote! {
                Some(#comment.into())
            }
        } else {
            quote! {
                None
            }
        };

        let mut type_of = if partial {
            quote! { SchemaType::infer_partial::<#value>() }
        } else {
            quote! { SchemaType::infer::<#value>() }
        };

        if let Some(default) = &self.args.default {
            if let Expr::Lit(lit) = &default {
                let lit_value = match &lit.lit {
                    Lit::Str(v) => quote! { LiteralValue::String(#v.into()) },
                    Lit::Int(v) => {
                        if v.suffix().starts_with('u') {
                            quote! { LiteralValue::Uint(#v) }
                        } else {
                            quote! { LiteralValue::Int(#v) }
                        }
                    }
                    Lit::Float(v) => {
                        if v.suffix() == "f32" {
                            quote! { LiteralValue::F32(#v) }
                        } else {
                            quote! { LiteralValue::F64(#v) }
                        }
                    }
                    Lit::Bool(v) => quote! { LiteralValue::Bool(#v) },
                    _ => unimplemented!(),
                };

                type_of = quote! { SchemaType::infer_with_default::<#value>(#lit_value) };
            }
        }

        quote! {
            SchemaField {
                name: Some(#name.into()),
                description: #description,
                type_of: #type_of,
                deprecated: #deprecated,
                hidden: #hidden,
                nullable: #nullable,
                ..Default::default()
            }
        }
    }

    pub fn get_serde_meta(&self) -> Option<TokenStream> {
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

        if meta.is_empty() {
            return None;
        }

        Some(quote! {
            #(#meta),*
        })
    }
}

impl<'l> ToTokens for Setting<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = &self.value_type;

        // Gather all attributes
        let mut attrs = vec![];

        if let Some(serde_meta) = self.get_serde_meta() {
            attrs.push(quote! { #[serde(#serde_meta)] });
        }

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }

        tokens.extend(quote! {
            #(#attrs)*
            pub #name: #value,
        });
    }
}
