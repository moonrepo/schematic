use super::setting::SettingArgs;
use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, GenericArgument, Lit, PathArguments, Type, TypePath};

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
    // Vec<item>
    NestedList {
        collection: &'l Ident,
        item: &'l GenericArgument,
        optional: bool,
        path: &'l TypePath,
    },
    // HashMap<key, value>
    NestedMap {
        collection: &'l Ident,
        key: &'l GenericArgument,
        optional: bool,
        path: &'l TypePath,
        value: &'l GenericArgument,
    },
    // config
    NestedValue {
        config: &'l Ident,
        optional: bool,
        path: &'l TypePath,
    },
    // value
    Value {
        optional: bool,
        value: &'l Type,
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

        let segment = path.path.segments.last().unwrap();
        let container = &segment.ident;

        match &segment.arguments {
            PathArguments::None => SettingType::NestedValue {
                path,
                config: container,
                optional,
            },
            PathArguments::AngleBracketed(args) => match container.to_string().as_str() {
                "Vec" | "HashSet" | "FxHashSet" | "BTreeSet" => SettingType::NestedList {
                    collection: container,
                    item: args.args.first().unwrap(),
                    optional,
                    path,
                },
                "HashMap" | "FxHashMap" | "BTreeMap" => SettingType::NestedMap {
                    collection: container,
                    key: args.args.first().unwrap(),
                    optional,
                    path,
                    value: args.args.last().unwrap(),
                },
                _ => panic!("Unsupported collection used with nested config."),
            },
            _ => panic!("Parens are not supported for nested config."),
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

        SettingType::Value { value, optional }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            SettingType::NestedValue { optional, .. } => *optional,
            SettingType::NestedList { optional, .. } => *optional,
            SettingType::NestedMap { optional, .. } => *optional,
            SettingType::Value { optional, .. } => *optional,
        }
    }

    pub fn get_inner_type(&self) -> Option<&'l Type> {
        match self {
            SettingType::Value { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn generate_default_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType::NestedList { .. } | SettingType::NestedMap { .. } => {
                quote! { Some(Default::default()) }
            }
            SettingType::NestedValue { config, .. } => {
                let partial_name = format_ident!("Partial{}", config);

                quote! { #partial_name::default_values(context)? }
            }
            SettingType::Value { value, .. } => {
                if let Some(expr) = args.default.as_ref() {
                    match expr {
                        Expr::Array(_) | Expr::Call(_) | Expr::Macro(_) | Expr::Tuple(_) => {
                            quote! { Some(#expr) }
                        }
                        Expr::Path(func) => quote! { #func(context) },
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Str(string) => quote! {
                                Some(
                                    #value::try_from(#string)
                                        .map_err(|e| schematic::ConfigError::InvalidDefault(e.to_string()))?
                                )
                            },
                            other => quote! { Some(#other) },
                        },
                        invalid => {
                            let name = name.to_string();
                            let info = format!("{:?}", invalid);

                            panic!("Unsupported default value for {name} ({info}). May only provide literals, primitives, arrays, or tuples.");
                        }
                    }
                } else {
                    quote! { Some(Default::default()) }
                }
            }
        }
    }

    pub fn get_finalize_value(&self) -> Option<TokenStream> {
        match self {
            SettingType::NestedList { .. } | SettingType::NestedMap { .. } => {
                Some(self.map_data(quote! { value.finalize(context)? }))
            }
            SettingType::NestedValue { .. } => {
                Some(self.map_data(quote! { data.finalize(context)? }))
            }
            SettingType::Value { .. } => None,
        }
    }

    pub fn get_from_partial_value(&self) -> TokenStream {
        match self {
            SettingType::NestedList { item, .. } => self.map_data(quote! {
                #item::from_partial(value)
            }),
            SettingType::NestedMap { value, .. } => self.map_data(quote! {
                #value::from_partial(value)
            }),
            SettingType::NestedValue { config, .. } => quote! {
                #config::from_partial(data)
            },
            SettingType::Value { .. } => quote! { data },
        }
    }

    pub fn get_merge_statement(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        if let SettingType::NestedValue { .. } = self {
            if args.merge.is_some() {
                panic!("Nested configs do not support `merge` unless wrapped in a collection.");
            }

            return quote! {
                self.#name = schematic::internal::merge_partial_setting(
                    self.#name.take(),
                    next.#name.take(),
                    context,
                )?;
            };
        };

        if let Some(func) = args.merge.as_ref() {
            quote! {
                self.#name = schematic::internal::merge_setting(
                    self.#name.take(),
                    next.#name.take(),
                    context,
                    #func,
                )?;
            }
        } else {
            quote! {
                if next.#name.is_some() {
                    self.#name = next.#name;
                }
            }
        }
    }

    pub fn get_validate_statement(&self, name: &Ident, args: &SettingArgs) -> Option<TokenStream> {
        let name_quoted = format!("{}", name);

        match self {
            SettingType::NestedList { .. } => Some(quote! {
                for (i, item) in setting.iter().enumerate() {
                    if let Err(nested_error) = item.validate_with_path(context, path.join_key(#name_quoted).join_index(i)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }
            }),
            SettingType::NestedMap { .. } => Some(quote! {
                for (key, value) in setting {
                    if let Err(nested_error) = value.validate_with_path(context, path.join_key(#name_quoted).join_key(key)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }
            }),
            SettingType::NestedValue { .. } => Some(quote! {
                if let Err(nested_error) = setting.validate_with_path(context, path.join_key(#name_quoted)) {
                    errors.push(schematic::ValidateErrorType::nested(nested_error));
                }
            }),
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

                    Some(quote! {
                        if let Err(error) = #func(setting, self, context) {
                            errors.push(schematic::ValidateErrorType::setting(
                                path.join_key(#name_quoted),
                                error,
                            ));
                        }
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn map_data(&self, mapped_data: TokenStream) -> TokenStream {
        match self {
            SettingType::NestedList { collection, .. } => {
                quote! {
                    {
                        let mut result = #collection::new();
                        for value in data {
                            result.push(#mapped_data);
                        }
                        result
                    }
                }
            }
            SettingType::NestedMap { collection, .. } => {
                quote! {
                    {
                        let mut result = #collection::new();
                        for (key, value) in data {
                            result.insert(key, #mapped_data);
                        }
                        result
                    }
                }
            }
            SettingType::NestedValue { .. } | SettingType::Value { .. } => {
                quote! { #mapped_data }
            }
        }
    }
}

impl<'l> ToTokens for SettingType<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            SettingType::NestedList {
                collection, item, ..
            } => {
                quote! { Option<#collection<<#item as schematic::Config>::Partial>> }
            }
            SettingType::NestedMap {
                collection,
                key,
                value,
                ..
            } => {
                quote! {
                    Option<#collection<#key, <#value as schematic::Config>::Partial>>
                }
            }
            SettingType::NestedValue { path, .. } => {
                quote! { Option<<#path as schematic::Config>::Partial> }
            }
            SettingType::Value { value, .. } => {
                quote! { Option<#value> }
            }
        })
    }
}
