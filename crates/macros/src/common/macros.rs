use crate::common::{Container, Field, SerdeMeta, TaggedFormat, Variant};
use crate::utils::extract_common_attrs;
use darling::ast::NestedMeta;
use darling::{FromDeriveInput, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct ContainerSerdeArgs {
    pub default: bool,
    pub deny_unknown_fields: bool,

    // struct
    pub rename: Option<String>,
    pub rename_all: Option<String>,
    pub rename_all_fields: Option<String>,

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
    pub partial: PartialAttr,
    #[cfg(feature = "env")]
    pub env_prefix: Option<String>,

    // serde
    pub rename: Option<String>,
    pub rename_all: Option<String>,
    pub rename_all_fields: Option<String>,
    pub serde: SerdeMeta,
}

#[derive(Default)]
pub struct PartialAttr {
    meta: Vec<NestedMeta>,
}

impl ToTokens for PartialAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs: Vec<_> = self.meta.iter().map(|m| m.to_token_stream()).collect();
        if !attrs.is_empty() {
            tokens.extend(quote! {#[#(#attrs),*]});
        }
    }
}

impl FromMeta for PartialAttr {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        Ok(Self {
            meta: items.to_vec(),
        })
    }
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
        let args = MacroArgs::from_derive_input(input).unwrap_or_default();
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
                    base_casing_format
                        .unwrap_or("camelCase")
                        .clone_into(&mut casing_format);

                    Container::NamedStruct {
                        fields: fields
                            .named
                            .iter()
                            .map(|f| {
                                let mut field = Field::from(f);
                                field.serde_args.inherit_from_container(&serde_args);
                                field.casing_format.clone_from(&casing_format);
                                #[cfg(feature = "env")]
                                field.env_prefix.clone_from(&args.env_prefix);
                                field
                            })
                            .collect::<Vec<_>>(),
                    }
                }
                Fields::Unnamed(fields) => {
                    base_casing_format
                        .unwrap_or("camelCase")
                        .clone_into(&mut casing_format);

                    Container::UnnamedStruct {
                        fields: fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(index, f)| {
                                let mut field = Field::from(f);
                                field.index = index;
                                field.serde_args.inherit_from_container(&serde_args);
                                field.casing_format.clone_from(&casing_format);
                                #[cfg(feature = "env")]
                                field.env_prefix.clone_from(&args.env_prefix);
                                field
                            })
                            .collect::<Vec<_>>(),
                    }
                }
                Fields::Unit => {
                    panic!("Unit structs are not supported.");
                }
            },
            Data::Enum(data) => {
                args.rename_all_fields
                    .as_deref()
                    .or(serde_args.rename_all_fields.as_deref())
                    .or(base_casing_format)
                    .unwrap_or("kebab-case")
                    .clone_into(&mut casing_format);

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
                            let mut field = Variant::from(variant);
                            field.casing_format.clone_from(&casing_format);
                            field.tagged_format.clone_from(&tagged_format);
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

    #[cfg(feature = "schema")]
    pub fn get_name(&self) -> String {
        match &self.args.rename {
            Some(local) => local.to_owned(),
            _ => {
                if let Some(serde) = &self.serde_args.rename {
                    serde.to_owned()
                } else {
                    self.name.to_string()
                }
            }
        }
    }

    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        match &self.type_of {
            Container::NamedStruct { .. } => {
                meta.push(quote! { default });

                if self.serde_args.deny_unknown_fields || !self.args.allow_unknown_fields {
                    meta.push(quote! { deny_unknown_fields });
                }
            }
            Container::UnnamedStruct { .. } => {
                meta.push(quote! { default });
            }
            Container::Enum { .. } => {
                match &self.args.serde.content {
                    Some(content) => {
                        meta.push(quote! { content = #content });
                    }
                    _ => {
                        if let Some(content) = &self.serde_args.content {
                            meta.push(quote! { content = #content });
                        }
                    }
                }

                match &self.args.serde.tag {
                    Some(tag) => {
                        meta.push(quote! { tag = #tag });
                    }
                    _ => {
                        if let Some(tag) = &self.serde_args.tag {
                            meta.push(quote! { tag = #tag });
                        }
                    }
                }

                if self.args.serde.untagged || self.serde_args.untagged {
                    meta.push(quote! { untagged });
                }
            }
        };

        match &self.args.serde.expecting {
            Some(expecting) => {
                meta.push(quote! { expecting = #expecting });
            }
            _ => {
                if let Some(expecting) = &self.serde_args.expecting {
                    meta.push(quote! { expecting = #expecting });
                }
            }
        }

        match &self.args.rename {
            Some(rename) => {
                meta.push(quote! { rename = #rename });
            }
            _ => {
                if let Some(rename) = &self.serde_args.rename {
                    meta.push(quote! { rename = #rename });
                }
            }
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
        let partial = &self.args.partial;
        attrs.push(quote! { #partial });

        attrs
    }

    pub fn is_untagged(&self) -> bool {
        self.args.serde.untagged || self.serde_args.untagged
    }
}
