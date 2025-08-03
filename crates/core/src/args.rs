use crate::utils::to_type_string;
use darling::ast::NestedMeta;
use darling::{FromAttributes, FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::ops::Deref;
use syn::{Expr, Ident};

#[derive(Clone, Copy, Debug)]
pub enum SerdeIoDirection {
    From, // read / deserialize
    To,   // write / serialize
}

#[derive(Clone, Debug)]
pub enum SerdeTagFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
    // Special case for unit only enums
    Unit,
}

// #[serde(rename = "name")]
// #[serde(rename(deserialize = "de_name", serialize = "ser_name"))]
#[derive(Debug, Default, PartialEq)]
pub struct SerdeRenameArg {
    pub deserialize: Option<String>,
    pub serialize: Option<String>,
}

impl FromMeta for SerdeRenameArg {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self {
            deserialize: Some(value.into()),
            serialize: Some(value.into()),
        })
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(Default, FromMeta)]
        #[darling(default)]
        struct Rename {
            deserialize: Option<String>,
            serialize: Option<String>,
        }

        impl From<Rename> for SerdeRenameArg {
            fn from(value: Rename) -> Self {
                Self {
                    deserialize: value.deserialize,
                    serialize: value.serialize,
                }
            }
        }

        Rename::from_list(items).map(SerdeRenameArg::from)
    }
}

impl SerdeRenameArg {
    pub fn get_name(&self, dir: SerdeIoDirection) -> Option<&str> {
        match dir {
            SerdeIoDirection::From => self.deserialize.as_deref(),
            SerdeIoDirection::To => self.serialize.as_deref(),
        }
    }

    pub fn get_meta(&self, key: &str) -> TokenStream {
        match (self.deserialize.as_deref(), self.serialize.as_deref()) {
            (Some(de), Some(ser)) => {
                if de == ser {
                    quote! { #key = #de }
                } else {
                    quote! { #key(deserialize = #de, serialize = #ser) }
                }
            }
            (None, Some(ser)) => quote! { #key(serialize = #ser) },
            (Some(de), None) => quote! { #key(deserialize = #de) },
            _ => quote! {},
        }
    }
}

// #[serde()]
#[derive(Debug, Default, FromDeriveInput)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeContainerArgs {
    pub default: bool,
    pub deny_unknown_fields: bool,

    // struct
    pub rename: Option<SerdeRenameArg>,
    pub rename_all: Option<SerdeRenameArg>,
    pub rename_all_fields: Option<SerdeRenameArg>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[serde()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeFieldArgs {
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub default: bool,
    pub flatten: bool,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_deserializing_if: Option<String>,
    pub skip_serializing: bool,
    pub skip_serializing_if: Option<String>,

    // variant
    pub other: bool,
    pub untagged: bool,
}

// #[setting(partial)]
#[derive(Debug, Default)]
pub struct PartialArg {
    meta: Vec<NestedMeta>,
}

impl ToTokens for PartialArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs: Vec<_> = self.meta.iter().map(|m| m.to_token_stream()).collect();

        if !attrs.is_empty() {
            tokens.extend(quote! {#[#(#attrs),*]});
        }
    }
}

impl FromMeta for PartialArg {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        Ok(Self {
            meta: items.to_vec(),
        })
    }
}

// #[setting(nested)]
#[derive(Debug)]
pub enum NestedArg {
    Detect(bool),
    Ident(Ident),
}

impl NestedArg {
    pub fn is_nested(&self) -> bool {
        match self {
            NestedArg::Detect(inner) => *inner,
            NestedArg::Ident(_) => true,
        }
    }
}

impl FromMeta for NestedArg {
    // #[setting(nested)]
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Detect(true))
    }

    // #[setting(nested = true)]
    fn from_bool(value: bool) -> darling::Result<Self> {
        Ok(Self::Detect(value))
    }

    // #[setting(nested = NestedConfig)]
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Lit(lit) => Self::from_value(&lit.lit),
            Expr::Path(path) => {
                if path.path.segments.len() > 1 {
                    Err(darling::Error::custom(format!(
                        "Too many segments for `{}`, only a single identifier is allowed.",
                        to_type_string(path.to_token_stream())
                    )))
                } else {
                    Ok(Self::Ident(
                        path.path.segments.last().unwrap().ident.to_owned(),
                    ))
                }
            }
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
        .map_err(|e| e.with_span(expr))
    }
}

// #[setting(validate)]
#[derive(Debug)]
pub struct ValidateArg(Expr);

impl FromMeta for ValidateArg {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Call(_) | Expr::Path(_) => Ok(Self(expr.to_owned())),
            _ => Err(darling::Error::unexpected_expr_type(expr)),
        }
        .map_err(|e| e.with_span(expr))
    }
}

impl Deref for ValidateArg {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
