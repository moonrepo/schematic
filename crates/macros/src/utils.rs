use syn::AngleBracketedGenericArguments;

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
