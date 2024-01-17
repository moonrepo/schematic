use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{GenericArgument, PathArguments, Type, TypePath};

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

pub enum FieldValue<'l> {
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

impl<'l> FieldValue<'l> {
    pub fn nested(raw: &'l Type) -> FieldValue {
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
            PathArguments::None => Self::NestedValue {
                path,
                config: container,
                optional,
            },
            PathArguments::AngleBracketed(args) => {
                let name = container.to_string();

                if name.ends_with("Vec") || name.ends_with("Set") {
                    Self::NestedList {
                        collection: container,
                        item: args.args.first().unwrap(),
                        optional,
                        path,
                    }
                } else if name.ends_with("Map") {
                    Self::NestedMap {
                        collection: container,
                        key: args.args.first().unwrap(),
                        optional,
                        path,
                        value: args.args.last().unwrap(),
                    }
                } else {
                    panic!("Unsupported collection used with nested config.");
                }
            }
            _ => panic!("Parens are not supported for nested config."),
        }
    }

    pub fn value(raw: &'l Type) -> FieldValue {
        let mut optional = false;

        let value = if let Some(unwrapped_value) = unwrap_option(raw) {
            optional = true;
            unwrapped_value
        } else {
            raw
        };

        Self::Value { value, optional }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            Self::NestedValue { optional, .. } => *optional,
            Self::NestedList { optional, .. } => *optional,
            Self::NestedMap { optional, .. } => *optional,
            Self::Value { optional, .. } => *optional,
        }
    }

    pub fn get_inner_type(&self) -> Option<&'l Type> {
        match self {
            Self::Value { value, .. } => Some(value),
            _ => None,
        }
    }
}

// Only used for partials
impl<'l> ToTokens for FieldValue<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::NestedList {
                collection, item, ..
            } => {
                quote! { Option<#collection<<#item as schematic::Config>::Partial>> }
            }
            Self::NestedMap {
                collection,
                key,
                value,
                ..
            } => {
                quote! {
                    Option<#collection<#key, <#value as schematic::Config>::Partial>>
                }
            }
            Self::NestedValue { path, .. } => {
                quote! { Option<<#path as schematic::Config>::Partial> }
            }
            Self::Value { value, .. } => {
                quote! { Option<#value> }
            }
        })
    }
}
