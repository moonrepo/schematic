use crate::args::{SerdeIoDirection, SerdeRenameArg};
use convert_case::{Boundary, Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Lit, Meta, Path};

/// Resolve the container's `rename[_all]`, preferring the `config`/`schematic`
/// attribute over the `serde` one. Returns `None` when neither is set, as
/// there is no default case — a name is left exactly as it was written.
pub fn get_renamed_value(
    config: Option<&SerdeRenameArg>,
    serde: Option<&SerdeRenameArg>,
) -> Option<String> {
    let dir = SerdeIoDirection::From;

    config
        .and_then(|rename| rename.get_name(dir))
        .or_else(|| serde.and_then(|rename| rename.get_name(dir)))
        .map(|format| format.to_owned())
}

/// Apply a serde `rename_all` casing to a name. Mirrors what serde does to
/// the serialized key, so that derive-time names (schemas, settings,
/// validation paths) describe what is actually accepted.
/// The case names serde accepts for `rename_all`, and that `before_parse`
/// reuses. Kept in step with `schematic::internal::casing::format_case`, which does
/// the same conversion at runtime.
pub const CASE_FORMATS: [&str; 8] = [
    "lowercase",
    "UPPERCASE",
    "PascalCase",
    "camelCase",
    "snake_case",
    "SCREAMING_SNAKE_CASE",
    "kebab-case",
    "SCREAMING-KEBAB-CASE",
];

/// Panic when a case format isn't one serde recognizes. Takes the attribute
/// name so the message points at whichever one was misspelled.
pub fn validate_case_format(attr: &str, format: &str) {
    if !CASE_FORMATS.contains(&format) {
        panic!(
            "Unknown `{attr}` value `{format}`. Supported values are {}.",
            CASE_FORMATS.join(", ")
        );
    }
}

/// Apply a serde `rename_all` casing to a field or variant name, exactly as
/// serde does (a port of `RenameRule::apply_to_field` and `apply_to_variant`).
/// A variant is split before every uppercase letter, so `MacOS` becomes
/// `mac-o-s` in kebab case, which is what serde will accept.
pub fn format_case(format: &str, value: &str, is_variant: bool) -> String {
    validate_case_format("rename_all", format);

    if is_variant {
        format_variant_case(format, value)
    } else {
        format_field_case(format, value)
    }
}

fn format_variant_case(format: &str, variant: &str) -> String {
    match format {
        "PascalCase" => variant.to_owned(),
        "lowercase" => variant.to_ascii_lowercase(),
        "UPPERCASE" => variant.to_ascii_uppercase(),
        "camelCase" => match variant.chars().next() {
            Some(first) => first.to_ascii_lowercase().to_string() + &variant[first.len_utf8()..],
            None => String::new(),
        },
        "snake_case" => {
            let mut snake = String::new();

            for (i, ch) in variant.char_indices() {
                if i > 0 && ch.is_uppercase() {
                    snake.push('_');
                }

                snake.push(ch.to_ascii_lowercase());
            }

            snake
        }
        "SCREAMING_SNAKE_CASE" => format_variant_case("snake_case", variant).to_ascii_uppercase(),
        "kebab-case" => format_variant_case("snake_case", variant).replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => {
            format_variant_case("SCREAMING_SNAKE_CASE", variant).replace('_', "-")
        }
        _ => unreachable!("Validated above."),
    }
}

fn format_field_case(format: &str, field: &str) -> String {
    match format {
        "lowercase" | "snake_case" => field.to_owned(),
        "UPPERCASE" | "SCREAMING_SNAKE_CASE" => field.to_ascii_uppercase(),
        "PascalCase" => {
            let mut pascal = String::new();
            let mut capitalize = true;

            for ch in field.chars() {
                if ch == '_' {
                    capitalize = true;
                } else if capitalize {
                    pascal.push(ch.to_ascii_uppercase());
                    capitalize = false;
                } else {
                    pascal.push(ch);
                }
            }

            pascal
        }
        "camelCase" => {
            let pascal = format_field_case("PascalCase", field);

            match pascal.chars().next() {
                Some(first) => first.to_ascii_lowercase().to_string() + &pascal[first.len_utf8()..],
                None => String::new(),
            }
        }
        "kebab-case" => field.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => field.to_ascii_uppercase().replace('_', "-"),
        _ => unreachable!("Validated above."),
    }
}

/// Convert a name the way `schematic::internal::casing::format_case` converts input at
/// runtime for `#[config(before_parse)]`. That conversion treats an acronym as
/// one word, unlike serde, so `MacOS` becomes `mac-os` rather than `mac-o-s`,
/// and matching it as well keeps loosely cased input parsing.
pub fn format_loose_case(format: &str, value: &str) -> String {
    let case = match format {
        "lowercase" => return value.to_lowercase(),
        "UPPERCASE" => return value.to_uppercase(),
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "kebab-case" => Case::Kebab,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        _ => return value.to_owned(),
    };

    value
        .remove_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(case)
}

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

        let value = value.value();

        // A block comment arrives as a single multi-line value, where a
        // leading `*` continues the block rather than starting a list.
        // Line comments arrive one attribute per line, so a `*` there is
        // markdown and must survive.
        let block = value.contains('\n');

        for line in value.split('\n') {
            let mut line = line.trim();

            if block {
                line = line
                    .strip_prefix("* ")
                    .unwrap_or_else(|| if line == "*" { "" } else { line });
            }

            lines.push(line.to_owned());
        }
    }

    if lines.is_empty() {
        return None;
    }

    Some(lines.join("\n").trim().to_owned())
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
                    // Every value is consumed, not just `note`, otherwise
                    // parsing stops at the first key that isn't one
                    let value = meta.value()?.parse::<Lit>()?;

                    if meta.path.is_ident("note")
                        && let Lit::Str(value) = value
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

/// Convert a literal from an attribute into a schema `LiteralValue`, for the
/// literals a schema can represent.
pub fn impl_literal_value(lit: &Lit) -> Option<TokenStream> {
    Some(match lit {
        Lit::Str(v) => quote! { LiteralValue::String(#v.into()) },
        Lit::Int(v) => {
            // The suffix types the setting, not the literal, which is
            // always an `isize` or `usize`, so it is dropped
            let unsuffixed = syn::LitInt::new(v.base10_digits(), v.span());

            if v.suffix().starts_with('u') {
                quote! { LiteralValue::UInt(#unsuffixed) }
            } else {
                quote! { LiteralValue::Int(#unsuffixed) }
            }
        }
        Lit::Float(v) => {
            if v.suffix() == "f32" {
                quote! { LiteralValue::F32(#v) }
            } else {
                quote! { LiteralValue::F64(#v) }
            }
        }
        Lit::Bool(v) => quote! { LiteralValue::Bool(#v) },
        _ => return None,
    })
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
}
