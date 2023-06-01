use super::setting::SettingArgs;
use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, GenericArgument, Lit, PathArguments, PathSegment, Type, TypePath};

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

pub enum SettingType2<'l> {
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

impl<'l> SettingType2<'l> {
    pub fn nested(raw: &Type) -> SettingType2 {
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
            PathArguments::None => SettingType2::NestedValue {
                path,
                config: container,
                optional,
            },
            PathArguments::AngleBracketed(args) => match container.to_string().as_str() {
                "Vec" | "HashSet" | "FxHashSet" | "BTreeSet" => SettingType2::NestedList {
                    collection: container,
                    item: args.args.first().unwrap(),
                    optional,
                    path,
                },
                "HashMap" | "FxHashMap" | "BTreeMap" => SettingType2::NestedMap {
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

    pub fn value(raw: &Type) -> SettingType2 {
        let mut optional = false;

        let value = if let Some(unwrapped_value) = unwrap_option(raw) {
            optional = true;
            unwrapped_value
        } else {
            raw
        };

        SettingType2::Value { value, optional }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            SettingType2::NestedValue { optional, .. } => *optional,
            SettingType2::NestedList { optional, .. } => *optional,
            SettingType2::NestedMap { optional, .. } => *optional,
            SettingType2::Value { optional, .. } => *optional,
        }
    }

    pub fn get_default_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType2::NestedList { .. } | SettingType2::NestedMap { .. } => {
                quote! { Some(Default::default()) }
            }
            SettingType2::NestedValue { config, .. } => {
                let partial_name = format_ident!("Partial{}", config);

                quote! { Some(#partial_name::default_values(context)?) }
            }
            SettingType2::Value { value, .. } => {
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

    pub fn get_default_value(&self, name: &Ident, args: &SettingArgs) -> Option<TokenStream> {
        match self {
            SettingType::Nested { collection, .. } => match collection {
                NestedType::None(id) if !self.is_optional() => {
                    let partial_name = format_ident!("Partial{}", id);

                    Some(quote! { Some(#partial_name::default_values(context)?) })
                }
                _ => None,
            },
            SettingType::Value { value, .. } => {
                if let Some(expr) = args.default.as_ref() {
                    return Some(match expr {
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
                    });
                };

                None
            }
        }
    }

    pub fn get_env_value(&self, args: &SettingArgs) -> Option<TokenStream> {
        match (&args.env, &args.parse_env) {
            (Some(env), Some(parse_env)) => Some(quote! {
                schematic::internal::parse_from_env_var(#env, #parse_env)?
            }),
            (Some(env), None) => Some(quote! {
                schematic::internal::default_from_env_var(#env)?
            }),
            _ => None,
        }
    }

    pub fn get_finalize_value(&self) -> Option<TokenStream> {
        match self {
            SettingType::Nested { collection, .. } => Some(match collection {
                NestedType::None(_) => {
                    quote! {
                        data.finalize(context)?
                    }
                }
                NestedType::Set(api, _) => {
                    if *api == "Vec" {
                        quote! {
                            {
                                let mut result = #api::with_capacity(data.len());
                                for v in data {
                                    result.push(v.finalize(context)?);
                                }
                                result
                            }
                        }
                    } else {
                        quote! {
                            {
                                let mut result = #api::new();
                                for v in data {
                                    result.insert(v.finalize(context)?);
                                }
                                result
                            }
                        }
                    }
                }
                NestedType::Map(api, _, _) => {
                    quote! {
                        {
                            let mut result = #api::new();
                            for (k, v) in data {
                                result.insert(k, v.finalize(context)?);
                            }
                            result
                        }
                    }
                }
            }),
            _ => None,
        }
    }

    pub fn get_from_value(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        match self {
            SettingType::Nested {
                collection,
                optional,
                ..
            } => {
                let statement = match collection {
                    NestedType::None(id) => {
                        quote! {
                             #id::from_partial(data)
                        }
                    }
                    NestedType::Set(_, item) => {
                        quote! {
                            data
                                .into_iter()
                                .map(#item::from_partial)
                                .collect::<_>()
                        }
                    }
                    NestedType::Map(_, _, value) => {
                        quote! {
                            data
                                .into_iter()
                                .map(|(k, v)| (k, #value::from_partial(v)))
                                .collect::<_>()
                        }
                    }
                };

                if *optional {
                    quote! {
                        if let Some(data) = partial.#name {
                            Some(#statement)
                        } else {
                            None
                        }
                    }
                } else {
                    quote! {
                        {
                            let data = partial.#name.unwrap_or_default();
                            #statement
                        }
                    }
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

    pub fn get_merge_statement(&self, name: &Ident, args: &SettingArgs) -> TokenStream {
        // Nested values require special partial merging
        if let SettingType::Nested {
            // However this only applies to direct types and
            // not those wrapped in a collection
            collection: NestedType::None(_),
            ..
        } = self
        {
            if args.merge.is_some() {
                panic!("Nested configs do not support `merge` unless wrapped in a collection.");
            }

            return quote! {
                self.#name = schematic::internal::merge_partial_settings(
                    self.#name.take(),
                    next.#name.take(),
                    context,
                )?;
            };
        };

        // Everything elses uses basic merging
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
            SettingType::Nested { collection, .. } => match collection {
                NestedType::None(_) => Some(quote! {
                    if let Err(nested_error) = setting.validate_with_path(context, path.join_key(#name_quoted)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }),
                NestedType::Set(_, _) => Some(quote! {
                    for (i, item) in setting.iter().enumerate() {
                        if let Err(nested_error) = item.validate_with_path(context, path.join_key(#name_quoted).join_index(i)) {
                            errors.push(schematic::ValidateErrorType::nested(nested_error));
                        }
                    }
                }),
                NestedType::Map(_, _, _) => Some(quote! {
                    for (key, value) in setting {
                        if let Err(nested_error) = value.validate_with_path(context, path.join_key(#name_quoted).join_key(key)) {
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
