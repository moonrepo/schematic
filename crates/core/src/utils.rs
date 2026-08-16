use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, Path};

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

    [
        // Lints
        "allow",
        "expect",
        "warn",
        // Docs
        "deprecated",
        "doc",
        // Compilation
        "cfg",
        "default",
        "non_exhaustive",
    ]
    .into_iter()
    .any(|n| path.is_ident(n))
}

/// Extract the doc comment from a list of attributes, as a single
/// block of text, with list items preserved on their own lines.
pub fn extract_comment(attrs: &[Attribute]) -> Option<String> {
    let mut lines = vec![];

    for attr in attrs {
        let Meta::NameValue(meta) = &attr.meta else {
            continue;
        };

        if !meta.path.is_ident("doc") {
            continue;
        }

        let Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) = &meta.value
        else {
            continue;
        };

        for line in value.value().split('\n') {
            let line = line.trim();

            // Preserve list items as their own line
            if line.starts_with("* ") || line.starts_with("- ") {
                lines.push(format!("\n{line}"));
            } else {
                lines.push(line.to_owned());
            }
        }
    }

    if lines.is_empty() {
        return None;
    }

    Some(lines.join(" ").trim().to_owned())
}

/// Extract the deprecated message from a list of attributes. Returns an
/// empty string when deprecated without a message.
pub fn extract_deprecated(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !get_meta_path(&attr.meta).is_ident("deprecated") {
            continue;
        }

        match &attr.meta {
            // #[deprecated]
            Meta::Path(_) => {
                return Some(String::new());
            }
            // #[deprecated = "message"]
            Meta::NameValue(meta) => {
                if let Expr::Lit(lit) = &meta.value {
                    match &lit.lit {
                        Lit::Bool(value) => {
                            if value.value() {
                                return Some(String::new());
                            }
                        }
                        Lit::Str(value) => {
                            return Some(value.value().trim().to_owned());
                        }
                        _ => {}
                    };
                }
            }
            // #[deprecated(since = "", note = "message")]
            Meta::List(_) => {
                let mut message = String::new();

                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("note")
                        && let Ok(value) = meta.value()
                        && let Ok(Lit::Str(value)) = value.parse::<Lit>()
                    {
                        message = value.value().trim().to_owned();
                    }

                    Ok(())
                });

                return Some(message);
            }
        };
    }

    None
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
