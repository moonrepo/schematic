// Thanks to confique for the implementation:
// https://github.com/LukasKalbertodt/confique/blob/main/macro/src/util.rs
pub fn unwrap_option(ty: &syn::Type) -> Option<&syn::Type> {
    let ty = match ty {
        syn::Type::Path(path) => path,
        _ => return None,
    };

    if ty.qself.is_some() || ty.path.leading_colon.is_some() {
        return None;
    }

    let valid_paths = [
        &["Option"] as &[_],
        &["std", "option", "Option"],
        &["core", "option", "Option"],
    ];

    if !valid_paths
        .iter()
        .any(|vp| ty.path.segments.iter().map(|s| &s.ident).eq(*vp))
    {
        return None;
    }

    let args = match &ty.path.segments.last().unwrap().arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    if args.args.len() != 1 {
        return None;
    }

    match &args.args[0] {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}
