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
    pub casing_format: String,
    pub name: &'l Ident,
    pub type_of: Container<'l>,
}

impl<'l> Macro<'l> {
    pub fn from(input: &'l DeriveInput) -> Self {
        let args = MacroArgs::from_derive_input(input).expect("Failed to parse arguments.");
        let serde_args = ContainerSerdeArgs::from_derive_input(input).unwrap_or_default();

        let base_casing_format = args
            .rename_all
            .as_deref()
            .or(serde_args.rename_all.as_deref());

        #[allow(unused_assignments)]
        let mut casing_format = String::new();

        let config_type = match &input.data {
            Data::Struct(data) => match &data.fields {
                Fields::Named(fields) => {
                    casing_format = base_casing_format.unwrap_or("camelCase").to_owned();

                    Container::NamedStruct {
                        fields: fields
                            .named
                            .iter()
                            .map(|f| {
                                let mut field = Field::from(f);
                                field.casing_format = casing_format.clone();
                                field.env_prefix = args.env_prefix.clone();
                                field
                            })
                            .collect::<Vec<_>>(),
                    }
                }
                Fields::Unnamed(_) => {
                    panic!("Unnamed structs are not supported.");
                }
                Fields::Unit => {
                    panic!("Unit structs are not supported.");
                }
            },
            Data::Enum(data) => {
                casing_format = base_casing_format.unwrap_or("kebab-case").to_owned();

                let tagged_format = {
                    if args.serde.untagged || serde_args.untagged {
                        TaggedFormat::Untagged
                    } else {
                        match (
                            args.serde.tag.as_ref().or(serde_args.tag.as_ref()),
                            args.serde.content.as_ref().or(serde_args.content.as_ref()),
                        ) {
                            (Some(tag), Some(content)) => {
                                TaggedFormat::Adjacent(tag.to_owned(), content.to_owned())
                            }
                            (Some(tag), None) => TaggedFormat::Internal(tag.to_owned()),
                            _ => TaggedFormat::External,
                        }
                    }
                };

                Container::Enum {
                    variants: data
                        .variants
                        .iter()
                        .map(|variant| {
                            if matches!(variant.fields, Fields::Named(_)) {
                                panic!("Named enum variants are not supported.");
                            }

                            let mut field = Variant::from(variant);
                            field.casing_format = casing_format.clone();
                            field.tagged_format = tagged_format.clone();
                            field
                        })
                        .collect(),
                }
            }
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
            casing_format,
        }
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

    pub fn get_name(&self) -> String {
        if let Some(local) = &self.args.rename {
            local.to_owned()
        } else if let Some(serde) = &self.serde_args.rename {
            serde.to_owned()
        } else {
            self.name.to_string()
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

        let rename_all = &self.casing_format;

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
