use crate::common::FieldSerdeArgs;
use crate::utils::{
    extract_comment, extract_common_attrs, extract_deprecated, format_case, get_meta_path,
    map_option_quote,
};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Fields, Variant as NativeVariant};

// #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(variant))]
pub struct VariantArgs {
    pub fallback: bool,
    pub value: Option<String>,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub default: bool,
    pub serde_args: FieldSerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: String,
}

impl<'l> Variant<'l> {
    pub fn from<'n>(variant: &'n NativeVariant, format: &str) -> Variant<'n> {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap_or_default();
        let serde_args = FieldSerdeArgs::from_attributes(&variant.attrs).unwrap_or_default();

        if args.fallback {
            if let Fields::Unnamed(fields) = &variant.fields {
                if fields.unnamed.len() != 1 {
                    panic!("Only 1 unnamed field is supported for `fallback`.");
                }
            } else {
                panic!("Only unnamed tuple variants are supported for `fallback`.");
            }

            if args.value.is_some() {
                panic!("`value` is not supported for `fallback`.");
            }
        } else if !matches!(variant.fields, Fields::Unit) {
            panic!("Only unit variants are supported.");
        }

        let value = if args.fallback {
            String::new()
        } else if let Some(v) = &args.value {
            v.to_owned()
        } else if let Some(v) = &serde_args.rename {
            v.to_owned()
        } else {
            format_case(format, variant.ident.to_string().as_str(), true)
        };

        let attrs = extract_common_attrs(&variant.attrs);

        Variant {
            default: attrs
                .iter()
                .any(|v| get_meta_path(&v.meta).is_ident("default")),
            attrs,
            name: &variant.ident,
            value,
            args,
            serde_args,
        }
    }

    pub fn get_display_fmt(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        if self.args.fallback {
            quote! {
                Self::#name(fallback) => fallback,
            }
        } else {
            quote! {
                Self::#name => #value,
            }
        }
    }

    pub fn get_from_str(&self) -> TokenStream {
        let name = &self.name;
        let value = &self.value;

        if self.args.fallback {
            quote! {
                fallback => Self::#name(
                    fallback.try_into().map_err(|_| {
                        schematic::ConfigError::EnumInvalidFallback(fallback.to_string())
                    })?
                ),
            }
        } else if let Some(alias) = &self.serde_args.alias {
            quote! {
                #value | #alias => Self::#name,
            }
        } else {
            quote! {
                #value => Self::#name,
            }
        }
    }

    pub fn get_schema_type(&self) -> TokenStream {
        let name = self.name.to_string();
        let description = map_option_quote("description", extract_comment(&self.attrs));
        let deprecated = map_option_quote("deprecated", extract_deprecated(&self.attrs));

        let inner_schema = if self.args.fallback {
            quote! {
                Schema::string(StringType::default())
            }
        } else {
            let value = &self.value;

            quote! {
                Schema::literal_value(LiteralValue::String(#value.into()))
            }
        };

        quote! {
            SchemaField {
                name: #name.into(),
                schema: #inner_schema,
                #description
                #deprecated
                ..Default::default()
            }
        }
    }

    pub fn get_unit_name(&self) -> TokenStream {
        let name = &self.name;

        if self.args.fallback {
            quote! {
                Self::#name(Default::default())
            }
        } else {
            quote! {
                Self::#name
            }
        }
    }
}
