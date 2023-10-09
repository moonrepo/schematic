use crate::common::{Container, Field, SerdeMeta, TaggedFormat, Variant};
use crate::utils::extract_common_attrs;
use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct ContainerSerdeArgs {
    // struct
    pub rename: Option<String>,
    pub rename_all: Option<String>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[config()], #[schematic()]
#[derive(FromDeriveInput, Default)]
#[darling(
    default,
    attributes(config, schematic),
    supports(struct_named, enum_any)
)]
pub struct MacroArgs {
    // config
    pub allow_unknown_fields: bool,
    pub context: Option<ExprPath>,
    pub env_prefix: Option<String>,
    pub file: Option<String>,

    // serde
    pub rename: Option<String>,
    pub rename_all: Option<String>,
    pub serde: SerdeMeta,
}

pub struct Macro<'l> {
    pub args: MacroArgs,
    pub serde_args: ContainerSerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub type_of: Container<'l>,
}

impl<'l> Macro<'l> {
    pub fn from(input: &'l DeriveInput) -> Self {
        let args = MacroArgs::from_derive_input(input).expect("Failed to parse arguments.");
        let serde_args = ContainerSerdeArgs::from_derive_input(input).unwrap_or_default();

        let config_type = match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(fields) => Container::NamedStruct {
                    fields: fields.named.iter().map(Field::from).collect::<Vec<_>>(),
                },
                Fields::Unnamed(_) => {
                    panic!("Unnamed structs are not supported.");
                }
                Fields::Unit => {
                    panic!("Unit structs are not supported.");
                }
            },
            Data::Enum(data) => Container::Enum {
                variants: data
                    .variants
                    .iter()
                    .map(|variant| {
                        if matches!(variant.fields, Fields::Named(_)) {
                            panic!("Named enum variants are not supported.");
                        }

                        Variant::from(variant)
                    })
                    .collect(),
            },
            Data::Union(_) => {
                panic!("Unions are not supported.");
            }
        };

        Self {
            args,
            serde_args,
            attrs: extract_common_attrs(&input.attrs),
            name: &input.ident,
            type_of: config_type,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self.type_of, Container::Enum { .. })
    }

    pub fn get_meta_struct(&self) -> TokenStream {
        let name = if let Some(rename) = &self.args.rename {
            rename.to_string()
        } else {
            format!("{}", self.name)
        };

        quote! {
            schematic::Meta {
                name: #name,
            }
        }
    }

    pub fn get_casing_format(&self) -> &str {
        self.args
            .rename_all
            .as_deref()
            .or(self.serde_args.rename_all.as_deref())
            .unwrap_or(if self.is_enum() {
                "kebab-case"
            } else {
                "camelCase"
            })
    }

    pub fn get_tagged_format(&self) -> TaggedFormat {
        if self.args.serde.untagged || self.serde_args.untagged {
            return TaggedFormat::Untagged;
        }

        match (
            self.args
                .serde
                .tag
                .as_ref()
                .or(self.serde_args.tag.as_ref()),
            self.args
                .serde
                .content
                .as_ref()
                .or(self.serde_args.content.as_ref()),
        ) {
            (Some(tag), Some(content)) => {
                TaggedFormat::Adjacent(tag.to_owned(), content.to_owned())
            }
            (Some(tag), None) => TaggedFormat::Internal(tag.to_owned()),
            _ => TaggedFormat::External,
        }
    }

    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        match &self.type_of {
            Container::NamedStruct { .. } => {
                meta.push(quote! { default });

                if !self.args.allow_unknown_fields {
                    meta.push(quote! { deny_unknown_fields });
                }
            }
            Container::Enum { .. } => {
                if let Some(content) = &self.args.serde.content {
                    meta.push(quote! { content = #content });
                } else if let Some(content) = &self.serde_args.content {
                    meta.push(quote! { content = #content });
                }

                if let Some(tag) = &self.args.serde.tag {
                    meta.push(quote! { tag = #tag });
                } else if let Some(tag) = &self.serde_args.tag {
                    meta.push(quote! { tag = #tag });
                }

                if self.args.serde.untagged || self.serde_args.untagged {
                    meta.push(quote! { untagged });
                }
            }
        };

        if let Some(expecting) = &self.args.serde.expecting {
            meta.push(quote! { expecting = #expecting });
        } else if let Some(expecting) = &self.serde_args.expecting {
            meta.push(quote! { expecting = #expecting });
        }

        if let Some(rename) = &self.args.rename {
            meta.push(quote! { rename = #rename });
        } else if let Some(rename) = &self.serde_args.rename {
            meta.push(quote! { rename = #rename });
        }

        let rename_all = self.get_casing_format();

        meta.push(quote! { rename_all = #rename_all });

        quote! {
            #(#meta),*
        }
    }

    pub fn get_partial_attrs(&self) -> Vec<TokenStream> {
        let serde_meta = self.get_serde_meta();
        let mut attrs = vec![quote! { #[serde(#serde_meta) ]}];

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }

        attrs
    }
}
