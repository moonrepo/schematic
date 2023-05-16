use super::setting::SettingArgs;
use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Type;
use syn::{Expr, GenericArgument, PathArguments, TypePath};

fn get_option_inner(path: &TypePath) -> Option<&TypePath> {
    let last_segment = path.path.segments.last().unwrap();

    if last_segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
        return None;
    };

    let GenericArgument::Type(arg) = &args.args[0] else {
        return None;
    };

    match &arg {
        Type::Path(t) => Some(t),
        _ => None,
    }
}

pub enum SettingType<'l> {
    Nested {
        // Raw original value type
        raw: &'l Type,

        // Inner raw path type
        raw_path: &'l TypePath,

        // Inner path type with `Option` unwrapped
        path: &'l TypePath,

        // Wrapped in `Option`
        optional: bool,
    },
    Value {
        // Raw original value type
        raw: &'l Type,

        // Inner value type with `Option` unwrapped
        value: &'l Type,

        // Wrapped in `Option`
        optional: bool,
    },
}

impl<'l> SettingType<'l> {
    pub fn nested(raw: &Type) -> SettingType {
        let mut optional = false;

        let Type::Path(raw_path) = raw else {
            panic!("Nested values may only be paths/type references.");
        };

        let path = if let Some(unwrapped_path) = get_option_inner(raw_path) {
            optional = true;
            unwrapped_path
        } else {
            raw_path
        };

        SettingType::Nested {
            raw,
            raw_path,
            path,
            optional,
        }
    }

    pub fn value(raw: &Type) -> SettingType {
        let mut optional = false;

        let value = if let Some(unwrapped_value) = unwrap_option(raw) {
            optional = true;
            unwrapped_value
        } else {
            raw
        };

        SettingType::Value {
            raw,
            value,
            optional: false,
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            SettingType::Nested { optional, .. } => *optional,
            SettingType::Value { optional, .. } => *optional,
        }
    }

    pub fn get_default_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType::Nested { path, .. } => {
                let last = path.path.segments.last().unwrap();

                if matches!(last.arguments, PathArguments::None) {
                    let partial_name = format_ident!("Partial{}", last.ident);

                    // Struct
                    quote! { Some(#partial_name::default_values()) }
                } else {
                    // Vec<Struct>, HashMap<_, Struct>, ...
                    quote! { Some(Default::default()) }
                }
            }
            SettingType::Value {
                optional, value, ..
            } => {
                if let Some(func) = args.default_fn.as_ref() {
                    return quote! { Some(#func()) };
                };

                if let Some(string) = args.default_str.as_ref() {
                    return quote! {
                        Some(
                            #value::try_from(#string)
                                .map_err(|e| schematic::ConfigError::InvalidDefault(e.to_string()))?
                        )
                    };
                };

                if let Some(expr) = args.default.as_ref() {
                    return match expr {
                        Expr::Array(_) | Expr::Call(_) | Expr::Lit(_) | Expr::Tuple(_) => {
                            quote! { Some(#expr) }
                        }
                        invalid => {
                            let name = name.to_string();
                            let info = format!("{:?}", invalid);

                            panic!("Unsupported default value for {name} ({info}). May only provide literals, arrays, or tuples. Use `default_fn` or `default_str` for more complex defaults.");
                        }
                    };
                };

                quote! { None }
            }
        }
    }

    pub fn get_from_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType::Nested { path, .. } => {
                // TODO
                quote! { None }
            }
            SettingType::Value { optional, .. } => {
                // Reset extendable values since we don't have the entire resolved list
                if args.extend {
                    quote! { Default::default() }

                    // Use optional values as-is as they're already wrapped in `Option`
                } else if *optional {
                    quote! { partial.#name }

                    // Otherwise unwrap the resolved value or use the type default
                } else {
                    quote! { partial.#name.unwrap_or_default() }
                }
            }
        }
    }

    pub fn get_validate_statement(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        let name_quoted = format!("{}", name);

        match self {
            SettingType::Nested { .. } => {
                quote! {
                    if let Err(nested_error) = setting.validate_with_path(path.join_key(#name_quoted)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }
            }
            SettingType::Value { .. } => {
                if let Some(expr) = args.validate.as_ref() {
                    let func = match expr {
                        // func(arg)()
                        Expr::Call(func) => quote! { #func },
                        // func()
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
                    quote! {}
                }
            }
        }
    }
}
