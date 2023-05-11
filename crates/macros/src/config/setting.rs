use darling::FromAttributes;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprLit, Field, Lit, Meta, Type};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct SettingArgs {
    nested: bool,
}

pub struct Setting<'l> {
    args: SettingArgs,
    comment: Option<String>,
    name: &'l Ident,
    value: &'l Type,
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        Setting {
            args: SettingArgs::from_attributes(&field.attrs).unwrap_or_default(),
            comment: extract_comment(field),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
        }
    }

    pub fn get_nested_struct_name(&self) -> Ident {
        match &self.value {
            Type::Path(path) => {
                let segments = &path.path.segments;
                let last_segment = segments.last().unwrap();

                format_ident!("Partial{}", &last_segment.ident)
            }
            _ => panic!("Only structs are supported for nested settings."),
        }
    }
}

impl<'l> ToTokens for Setting<'l> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let comment = if let Some(cmt) = &self.comment {
            quote! { #[doc = #cmt] }
        } else {
            quote! {}
        };

        if self.args.nested {
            let ident = self.get_nested_struct_name();

            tokens.extend(quote! {
                #comment
                pub #name: Option<#ident>,
            });
        } else {
            let value = self.value;

            tokens.extend(quote! {
                #comment
                pub #name: Option<#value>,
            });
        }
    }
}

fn extract_comment(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("doc") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &meta.value
                {
                    return Some(value.value().trim().to_string());
                }
            }
        }
    }

    None
}
