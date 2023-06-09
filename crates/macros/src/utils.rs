use convert_case::{Case, Casing};
use syn::{AngleBracketedGenericArguments, Attribute, Expr, Meta};

pub fn format_case(format: &str, value: &str) -> String {
    let case = match format {
        "lowercase" => Case::Lower,
        "UPPERCASE" => Case::Upper,
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        _ => Case::Kebab,
    };

    value.to_case(case)
}

pub fn preserve_str_literal(meta: &Meta) -> darling::Result<Expr> {
    match meta {
        Meta::Path(_) => Err(darling::Error::unsupported_format("path").with_span(meta)),
        Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
        Meta::NameValue(nv) => Ok(nv.value.clone()),
    }
}

pub fn extract_common_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    let preserve = ["allow", "deprecated", "doc", "warn"];

    attrs
        .iter()
        .filter(|a| {
            let path = match &a.meta {
                Meta::Path(path) => path,
                Meta::List(list) => &list.path,
                Meta::NameValue(nv) => &nv.path,
            };

            preserve.iter().any(|n| path.is_ident(n))
        })
        .collect()
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
