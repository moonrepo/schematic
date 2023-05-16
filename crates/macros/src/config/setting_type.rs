use super::setting::SettingArgs;
use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, GenericArgument, PathArguments, TypePath};
use syn::{PathSegment, Type};

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

        // Inner path type with `Option` unwrapped
        path: &'l TypePath,

        // Path type wrapped in a collection
        collection: NestedType<'l>,

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
            collection: NestedType::from(path.path.segments.last().unwrap()),
            raw,
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
            optional,
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            SettingType::Nested { optional, .. } => *optional,
            SettingType::Value { optional, .. } => *optional,
        }
    }

    pub fn get_inner_type(&self) -> Option<&'l Type> {
        match self {
            SettingType::Nested { .. } => None,
            SettingType::Value { value, .. } => Some(value),
        }
    }

    pub fn get_default_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType::Nested { collection, .. } => match collection {
                NestedType::None(id) => {
                    let partial_name = format_ident!("Partial{}", id);

                    quote! { Some(#partial_name::default_values()?) }
                }
                _ => quote! { None },
            },
            SettingType::Value { value, .. } => {
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
            SettingType::Nested {
                collection,
                optional,
                ..
            } => {
                let callback = match collection {
                    NestedType::None(id) => {
                        quote! { #id::from_partial }
                    }
                    NestedType::Set(_, item) => {
                        quote! {
                            |data| {
                                data
                                .into_iter()
                                .map(#item::from_partial)
                                .collect::<_>()
                            }
                        }
                    }
                    NestedType::Map(_, _, value) => {
                        quote! {
                            |data| {
                                data
                                .into_iter()
                                .map(|(k, v)| (k, #value::from_partial(v)))
                                .collect::<_>()
                            }
                        }
                    }
                };

                if *optional {
                    quote! { partial.#name.map(#callback) }
                } else {
                    quote! { partial.#name.map(#callback).unwrap() }
                }
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

    pub fn get_validate_statement(&self, name: &Ident, args: &SettingArgs) -> Option<TokenStream> {
        let name_quoted = format!("{}", name);

        match self {
            SettingType::Nested { collection, .. } => match collection {
                NestedType::None(_) => Some(quote! {
                    if let Err(nested_error) = setting.validate_with_path(path.join_key(#name_quoted)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }),
                NestedType::Set(_, _) => Some(quote! {
                    for (i, item) in setting.iter().enumerate() {
                        if let Err(nested_error) = item.validate_with_path(path.join_key(#name_quoted).join_index(i)) {
                            errors.push(schematic::ValidateErrorType::nested(nested_error));
                        }
                    }
                }),
                NestedType::Map(_, _, _) => Some(quote! {
                    for (key, value) in setting {
                        if let Err(nested_error) = value.validate_with_path(path.join_key(#name_quoted).join_key(key)) {
                            errors.push(schematic::ValidateErrorType::nested(nested_error));
                        }
                    }
                }),
            },
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
                        if let Err(error) = #func(setting, self) {
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
}

impl<'l> ToTokens for SettingType<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SettingType::Nested {
                path, collection, ..
            } => {
                tokens.extend(match collection {
                    NestedType::None(_) => {
                        quote! { Option<<#path as schematic::Config>::Partial> }
                    }
                    NestedType::Set(container, item) => {
                        quote! { Option<#container<<#item as schematic::Config>::Partial>> }
                    }
                    NestedType::Map(container, key, value) => {
                        quote! {
                            Option<#container<#key, <#value as schematic::Config>::Partial>>
                        }
                    }
                });
            }
            SettingType::Value { value, .. } => {
                tokens.extend(quote! { Option<#value> });
            }
        }
    }
}

pub enum NestedType<'l> {
    // Struct
    None(&'l Ident),
    // Vec<Struct>, HashSet<Struct>, ...
    Set(&'l Ident, &'l GenericArgument),
    // HashMap<_, Struct>, ...
    Map(&'l Ident, &'l GenericArgument, &'l GenericArgument),
}

impl<'l> NestedType<'l> {
    pub fn from(segment: &PathSegment) -> NestedType {
        let container = &segment.ident;

        match &segment.arguments {
            // Struct
            PathArguments::None => NestedType::None(container),
            // Vec<Struct>, HashMap<_, Struct>, ...
            PathArguments::AngleBracketed(args) => match container.to_string().as_str() {
                "Vec" | "HashSet" | "FxHashSet" | "BTreeSet" => {
                    let item = args.args.first().unwrap();

                    NestedType::Set(container, item)
                }
                "HashMap" | "FxHashMap" | "BTreeMap" => {
                    let key = args.args.first().unwrap();
                    let value = args.args.last().unwrap();

                    NestedType::Map(container, key, value)
                }
                _ => panic!("Unsupported collection used with nested config."),
            },
            // ...
            _ => panic!("Parens are not supported for nested config."),
        }
    }
}
