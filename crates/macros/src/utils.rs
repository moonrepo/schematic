use convert_case::{Boundary, Case, Casing};
use quote::{format_ident, quote, ToTokens};
use syn::{AngleBracketedGenericArguments, Attribute, Expr, ExprLit, Lit, Meta, Path};

pub fn format_case(format: &str, value: &str, is_variant: bool) -> String {
    let case = match format {
        "lowercase" => {
            return value.to_lowercase();
        }
        "UPPERCASE" => {
            return value.to_uppercase();
        }
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        _ => Case::Kebab,
    };

    value
        .from_case(if is_variant {
            Case::Pascal
        } else {
            Case::Snake
        })
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(case)
}

pub fn preserve_str_literal(meta: &Meta) -> darling::Result<Expr> {
    match meta {
        Meta::Path(_) => Err(darling::Error::unsupported_format("path").with_span(meta)),
        Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
        Meta::NameValue(nv) => Ok(nv.value.clone()),
    }
}

pub fn get_meta_path(meta: &Meta) -> &Path {
    match meta {
        Meta::Path(path) => path,
        Meta::List(list) => &list.path,
        Meta::NameValue(nv) => &nv.path,
    }
}

pub fn extract_common_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    let preserve = ["allow", "default", "deprecated", "doc", "warn"];

    attrs
        .iter()
        .filter(|a| {
            let path = get_meta_path(&a.meta);

            preserve.iter().any(|n| path.is_ident(n))
        })
        .collect()
}

pub fn extract_comment(attrs: &[&Attribute]) -> Option<String> {
    let mut lines = vec![];

    for attr in attrs {
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("doc") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &meta.value
                {
                    for line in value.value().split('\n') {
                        lines.push(line.trim().replace("* ", "").replace(" * ", ""));
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

pub fn extract_deprecated(attrs: &[&Attribute]) -> Option<String> {
    for attr in attrs {
        match &attr.meta {
            Meta::NameValue(meta) => {
                if meta.path.is_ident("deprecated") {
                    if let Expr::Lit(lit) = &meta.value {
                        match &lit.lit {
                            Lit::Bool(value) => {
                                if value.value() {
                                    return Some(String::new()); // No message, handle in renderer
                                }
                            }
                            Lit::Str(value) => {
                                return Some(value.value().trim().to_owned());
                            }
                            _ => {}
                        };
                    }
                }
            }
            Meta::Path(_) => {
                if get_meta_path(&attr.meta).is_ident("deprecated") {
                    return Some(String::new()); // No message, handle in renderer
                }
            }
            _ => {}
        }
    }

    None
}

// Thanks to confique for the implementation:
// https://github.com/LukasKalbertodt/confique/blob/main/macro/src/util.rs
pub fn unwrap_path_type<'l>(
    ty: &'l syn::Type,
    lookups: &[&[&str]],
) -> Option<&'l AngleBracketedGenericArguments> {
    let ty = match ty {
        syn::Type::Path(path) => path,
        _ => return None,
    };

    if ty.qself.is_some() || ty.path.leading_colon.is_some() {
        return None;
    }

    if !lookups
        .iter()
        .any(|vp| ty.path.segments.iter().map(|s| &s.ident).eq(*vp))
    {
        return None;
    }

    match &ty.path.segments.last().unwrap().arguments {
        syn::PathArguments::AngleBracketed(args) => Some(args),
        _ => None,
    }
}

pub fn unwrap_option(ty: &syn::Type) -> Option<&syn::Type> {
    let Some(args) = unwrap_path_type(
        ty,
        &[
            &["Option"],
            &["std", "option", "Option"],
            &["core", "option", "Option"],
        ],
    ) else {
        return None;
    };

    if args.args.len() != 1 {
        return None;
    }

    match &args.args[0] {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}

pub fn map_bool_quote(name: &str, value: bool) -> Option<proc_macro2::TokenStream> {
    if value {
        let id = format_ident!("{}", name);

        Some(quote! {
            #id: true,

        })
    } else {
        None
    }
}

pub fn map_option_quote<T: ToTokens>(
    name: &str,
    value: Option<T>,
) -> Option<proc_macro2::TokenStream> {
    if let Some(value) = value {
        let id = format_ident!("{}", name);

        Some(quote! {
            #id: Some(#value.into()),

        })
    } else {
        None
    }
}
