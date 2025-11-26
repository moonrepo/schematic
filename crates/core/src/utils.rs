use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, Meta, Path};

pub fn get_meta_path(meta: &Meta) -> &Path {
    match meta {
        Meta::Path(path) => path,
        Meta::List(list) => &list.path,
        Meta::NameValue(nv) => &nv.path,
    }
}

pub fn preserve_str_literal(meta: &Meta) -> darling::Result<Expr> {
    match meta {
        Meta::Path(_) => Err(darling::Error::unsupported_format("path").with_span(meta)),
        Meta::List(_) => Err(darling::Error::unsupported_format("list").with_span(meta)),
        Meta::NameValue(nv) => Ok(nv.value.clone()),
    }
}

pub fn is_inheritable_attribute(attr: &Attribute) -> bool {
    let path = get_meta_path(&attr.meta);

    ["allow", "default", "deprecated", "doc", "warn"]
        .into_iter()
        .any(|n| path.is_ident(n))
}

pub fn to_type_string(ts: TokenStream) -> String {
    format!("{ts}")
        .replace(" :: ", "::")
        .replace(" , ", ", ")
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" <", "<")
        .replace(" > ", ">")
        .replace("> ", ">")
        .replace(" >", ">")
}

#[derive(Default)]
pub struct ImplResult {
    pub requires_internal: bool,
    pub no_value: bool,
    pub value: TokenStream,
}

impl ImplResult {
    pub fn skipped() -> Self {
        Self {
            no_value: true,
            ..Default::default()
        }
    }

    pub fn impl_struct_default(show: bool) -> TokenStream {
        if show {
            quote! { ..Default::default() }
        } else {
            quote! {}
        }
    }

    pub fn impl_use_internal(show: bool) -> TokenStream {
        if show {
            quote! { use schematic::internal::*; }
        } else {
            quote! {}
        }
    }
}
