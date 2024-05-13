use crate::utils::unwrap_option;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{GenericArgument, PathArguments, Type, TypePath};

fn get_inner_type<'a>(path: &'a TypePath, ident: &str) -> Option<&'a TypePath> {
    let last_segment = path.path.segments.last().unwrap();

    if last_segment.ident != ident {
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
        boxed: bool,
        collection: &'l Ident,
        item: &'l GenericArgument,
        optional: bool,
        path: &'l TypePath,
    },
    // HashMap<key, value>
    NestedMap {
        boxed: bool,
        collection: &'l Ident,
        key: &'l GenericArgument,
        optional: bool,
        path: &'l TypePath,
        value: &'l GenericArgument,
    },
    // config
    NestedValue {
        boxed: bool,
        config: &'l Ident,
        optional: bool,
        path: &'l TypePath,
    },
    // value
    Value {
        boxed: bool,
        optional: bool,
        value: &'l Type,
    },
}

impl<'l> FieldValue<'l> {
    pub fn nested(raw: &'l Type) -> FieldValue {
        let Type::Path(raw_path) = raw else {
            panic!("Nested values may only be paths/type references.");
        };

        let mut optional = false;
        let mut boxed = false;
        let mut path = raw_path;

        // Unwrap `Option`
        if let Some(unwrapped_path) = get_inner_type(path, "Option") {
            optional = true;
            path = unwrapped_path;
        }

        // Unwrap `Box`
        if let Some(unwrapped_path) = get_inner_type(path, "Box") {
            boxed = true;
            path = unwrapped_path;
        }

        let segment = path.path.segments.last().unwrap();
        let container = &segment.ident;

        match &segment.arguments {
            PathArguments::None => Self::NestedValue {
                boxed,
                path,
                config: container,
                optional,
            },
            PathArguments::AngleBracketed(args) => {
                let name = container.to_string();

                if name.ends_with("Vec") || name.ends_with("Set") {
                    Self::NestedList {
                        boxed,
                        collection: container,
                        item: args.args.first().unwrap(),
                        optional,
                        path,
                    }
                } else if name.ends_with("Map") {
                    Self::NestedMap {
                        boxed,
                        collection: container,
                        key: args.args.first().unwrap(),
                        optional,
                        path,
                        value: args.args.last().unwrap(),
                    }
                } else {
                    panic!("Unsupported collection {name} used with nested config.");
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

        Self::Value {
            boxed: false,
            value,
            optional,
        }
    }

    pub fn is_boxed(&self) -> bool {
        match self {
            Self::NestedValue { boxed, .. } => *boxed,
            Self::NestedList { boxed, .. } => *boxed,
            Self::NestedMap { boxed, .. } => *boxed,
            Self::Value { boxed, .. } => *boxed,
        }
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
        let inner = match self {
            Self::NestedList {
                collection, item, ..
            } => {
                quote! { #collection<<#item as schematic::Config>::Partial> }
            }
            Self::NestedMap {
                collection,
                key,
                value,
                ..
            } => {
                quote! {
                    #collection<#key, <#value as schematic::Config>::Partial>
                }
            }
            Self::NestedValue { path, .. } => {
                quote! { <#path as schematic::Config>::Partial }
            }
            Self::Value { value, .. } => {
                quote! { #value }
            }
        };

        // Boxes are ignored for the partial type,
        // and will only be used for the final type!
        // if self.is_boxed() {
        //     inner = quote! { Box<#inner> };
        // }

        tokens.extend(quote! { Option<#inner> })
    }
}
