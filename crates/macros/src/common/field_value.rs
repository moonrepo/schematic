use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{GenericArgument, PathArguments, Type};

fn is_collection_type(ident: &Ident) -> bool {
    let name = ident.to_string();

    name.ends_with("Vec") || name.ends_with("Set") || name.ends_with("Map")
}

#[derive(Debug, Default)]
pub struct TypeInfo {
    pub boxed: bool,
    pub optional: bool,
    pub config: Option<Ident>,
}

fn extract_inner_type<'a>(ty: &'a Type, info: &mut TypeInfo) -> &'a Type {
    // We don't need to traverse other types, just paths
    let Type::Path(type_path) = ty else {
        return ty;
    };

    // Extract the last segment of the path, for example `Option`,
    // instead of the full path `std::option::Option`
    let last_segment = type_path.path.segments.last().unwrap();

    // If a collecion type, return the path immediately, as we'll need
    // to extract inner information later on
    if is_collection_type(&last_segment.ident) {
        return ty;
    }

    // If a wrapper type, mark the information for later
    let mut nested = false;

    if last_segment.ident == "Option" {
        info.optional = true;
        nested = true;
    } else if last_segment.ident == "Box" {
        info.boxed = true;
        nested = true;
    }

    // If a nested type, drill down deeper to find the inner type
    if nested {
        if let PathArguments::AngleBracketed(args) = &last_segment.arguments
            && let GenericArgument::Type(inner_ty) = args.args.last().unwrap()
        {
            return extract_inner_type(inner_ty, info);
        }
    }
    // Otherwise we found the inner type, so extract the ident name
    else {
        info.config = Some(last_segment.ident.clone());
    }

    ty
}

#[derive(Debug)]
pub enum FieldValue<'l> {
    // Vec<item>
    NestedList {
        collection: &'l Ident,
        collection_info: TypeInfo,
        item: &'l Type,
        item_info: TypeInfo,
    },
    // HashMap<key, value>
    NestedMap {
        collection: &'l Ident,
        collection_info: TypeInfo,
        key: &'l Type,
        value: &'l Type,
        value_info: TypeInfo,
    },
    // config
    NestedValue {
        info: TypeInfo,
        value: &'l Type,
    },
    // value
    Value {
        info: TypeInfo,
        value: &'l Type,
    },
}

impl<'l> FieldValue<'l> {
    pub fn nested(raw: &'l Type) -> FieldValue<'l> {
        let mut outer_info = TypeInfo::default();
        let ty = extract_inner_type(raw, &mut outer_info);

        let Type::Path(ty_path) = ty else {
            panic!("Nested values may only be paths/type references.");
        };

        let segment = ty_path.path.segments.last().unwrap();
        let name = segment.ident.to_string();

        if name.ends_with("Vec") || name.ends_with("Set") {
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                panic!("Received a {name} without inner arguments!");
            };

            let Some(GenericArgument::Type(inner_ty)) = args.args.first() else {
                panic!("{name} item type must be a path!");
            };

            let mut inner_info = TypeInfo::default();
            let item = extract_inner_type(inner_ty, &mut inner_info);

            Self::NestedList {
                collection: &segment.ident,
                collection_info: outer_info,
                item,
                item_info: inner_info,
            }
        } else if name.ends_with("Map") {
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                panic!("Received a {name} without inner arguments!");
            };

            let Some(GenericArgument::Type(key_ty)) = args.args.first() else {
                panic!("{name} key type must be a path!");
            };

            let Some(GenericArgument::Type(value_ty)) = args.args.last() else {
                panic!("{name} value type must be a path!");
            };

            let mut inner_info = TypeInfo::default();
            let value = extract_inner_type(value_ty, &mut inner_info);

            Self::NestedMap {
                collection: &segment.ident,
                collection_info: outer_info,
                key: key_ty,
                value,
                value_info: inner_info,
            }
        } else {
            Self::NestedValue {
                info: outer_info,
                value: ty,
            }
        }
    }

    pub fn value(raw: &'l Type) -> FieldValue<'l> {
        let mut info = TypeInfo::default();
        let value = extract_inner_type(raw, &mut info);

        Self::Value { info, value }
    }

    pub fn is_outer_boxed(&self) -> bool {
        match self {
            Self::NestedValue { info, .. } => info.boxed,
            Self::NestedList {
                collection_info, ..
            } => collection_info.boxed,
            Self::NestedMap {
                collection_info, ..
            } => collection_info.boxed,
            Self::Value { info, .. } => info.boxed,
        }
    }

    pub fn is_outer_optional(&self) -> bool {
        match self {
            Self::NestedValue { info, .. } => info.optional,
            Self::NestedList {
                collection_info, ..
            } => collection_info.optional,
            Self::NestedMap {
                collection_info, ..
            } => collection_info.optional,
            Self::Value { info, .. } => info.optional,
        }
    }

    pub fn get_config_type(&self) -> &'l Type {
        match self {
            Self::NestedList { item, .. } => item,
            Self::NestedMap { value, .. } => value,
            Self::NestedValue { value, .. } => value,
            Self::Value { value, .. } => value,
        }
    }

    pub fn get_inner_type(&self) -> Option<&'l Type> {
        match self {
            Self::Value { value, .. } => Some(value),
            _ => None,
        }
    }
}

// Only used for partials!!!
impl ToTokens for FieldValue<'_> {
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
            Self::NestedValue { value, .. } => {
                quote! { <#value as schematic::Config>::Partial }
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
