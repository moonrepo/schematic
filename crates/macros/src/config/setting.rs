use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprLit, ExprPath, Field, Lit, Meta, Type};

#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct SettingArgs {
    default: Option<Expr>,
    default_fn: Option<ExprPath>,
    nested: bool,

    // serde
    rename: Option<String>,
    skip: Option<bool>,
}

impl SettingArgs {
    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        if let Some(rename) = &self.rename {
            meta.push(quote! { rename = #rename });
        }

        if self.skip.unwrap_or_default() {
            meta.push(quote! { skip });
        }

        meta.push(quote! { skip_serializing_if = "Option::is_none" });

        quote! {
            #(#meta),*
        }
    }
}

pub struct Setting<'l> {
    pub args: SettingArgs,
    pub comment: Option<String>,
    pub name: &'l Ident,
    pub value: &'l Type,
}

impl<'l> Setting<'l> {
    pub fn from(field: &Field) -> Setting {
        let args = SettingArgs::from_attributes(&field.attrs).unwrap_or_default();

        if args.nested {
            if args.default_fn.is_some() || args.default.is_some() {
                panic!("Cannot use `default` or `default_fn` with nested configs.");
            }

            // Others
        }

        if args.default_fn.is_some() && args.default.is_some() {
            panic!("Cannot provide both `default` and `default_fn`.");
        }

        Setting {
            args,
            comment: extract_comment(field),
            name: field.ident.as_ref().unwrap(),
            value: &field.ty,
        }
    }

    pub fn get_default_value(&self) -> TokenStream {
        if self.args.nested {
            let struct_name = self.get_nested_struct_name();

            return quote! { Some(#struct_name::default_values()) };
        };

        if let Some(func) = self.args.default_fn.as_ref() {
            return quote! { Some(#func()) };
        };

        let Some(expr) = self.args.default.as_ref() else {
            return quote! { None };
        };

        match expr {
            Expr::Array(_) | Expr::Lit(_) | Expr::Tuple(_) => {
                quote! { Some(#expr) }
            }
            // Strings are `Path` for some reason instead of `Lit`...
            Expr::Path(inner) => {
                let string = format!("{}", inner.to_token_stream());
                quote! { Some(#string.into()) }
            }
            invalid => {
                let name = self.name.to_string();
                let info = format!("{:?}", invalid);

                panic!("Unsupported default value for {name} ({info}). May only provide literals, arrays, or tuples. Use `default_fn` for more complex defaults.");
            }
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
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = self.value;

        let comment = if let Some(cmt) = &self.comment {
            quote! { #[doc = #cmt] }
        } else {
            quote! {}
        };

        let value = if self.args.nested {
            quote! { <#value as schematic::Config>::Partial }
        } else {
            quote! { #value }
        };

        let serde_meta = self.args.get_serde_meta();
        let attrs = quote! { #[serde(#serde_meta)] };

        tokens.extend(quote! {
            #comment
            #attrs
            pub #name: Option<#value>,
        });
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
                    return Some(value.value());
                }
            }
        }
    }

    None
}
