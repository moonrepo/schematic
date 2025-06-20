use proc_macro2::TokenStream;
use syn::{Attribute, Meta, Path};

pub fn get_meta_path(meta: &Meta) -> &Path {
    match meta {
        Meta::Path(path) => path,
        Meta::List(list) => &list.path,
        Meta::NameValue(nv) => &nv.path,
    }
}

pub fn is_inheritable_attribute(attr: &Attribute) -> bool {
    let path = get_meta_path(&attr.meta);

    ["allow", "default", "deprecated", "doc", "warn"]
        .into_iter()
        .any(|n| path.is_ident(n))
}

pub fn to_type_string(ts: TokenStream) -> String {
    format!("{}", ts)
        .replace(" :: ", "::")
        .replace(" , ", ", ")
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" <", "<")
        .replace(" > ", ">")
        .replace("> ", ">")
        .replace(" >", ">")
}
