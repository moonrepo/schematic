use crate::utils::{extract_common_attrs, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Fields, Variant as NativeVariant};

pub enum TaggedFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
}

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct VariantSerdeArgs {
    pub alias: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
}

// #[setting()], #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting, variant))]
pub struct VariantArgs {
    pub null: bool,

    // config
    pub default: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    pub validate: Option<Expr>,

    // serde
    pub rename: Option<String>,
    pub skip: bool,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub serde_args: VariantSerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l NativeVariant,
}

impl<'l> Variant<'l> {
    pub fn from(var: &NativeVariant) -> Variant {
        Variant {
            args: VariantArgs::from_attributes(&var.attrs).unwrap_or_default(),
            serde_args: VariantSerdeArgs::from_attributes(&var.attrs).unwrap_or_default(),
            attrs: extract_common_attrs(&var.attrs),
            name: &var.ident,
            value: var,
        }
    }

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn get_name(&self, casing_format: Option<&str>) -> String {
        if let Some(local) = &self.args.rename {
            local.to_owned()
        } else if let Some(serde) = &self.serde_args.rename {
            serde.to_owned()
        } else if let Some(format) = casing_format {
            format_case(format, &self.name.to_string(), true)
        } else {
            self.name.to_string()
        }
    }

    pub fn get_serde_meta(&self) -> Option<TokenStream> {
        let mut meta = vec![];

        if let Some(alias) = &self.serde_args.alias {
            meta.push(quote! { alias = #alias });
        }

        if let Some(rename) = &self.args.rename {
            meta.push(quote! { rename = #rename });
        } else if let Some(rename) = &self.serde_args.rename {
            meta.push(quote! { rename = #rename });
        }

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
        }

        if meta.is_empty() {
            return None;
        }

        Some(quote! {
            #(#meta),*
        })
    }

    pub fn generate_schema_type(
        &self,
        casing_format: &str,
        tagged_format: &TaggedFormat,
    ) -> TokenStream {
        let name = self.get_name(Some(casing_format));
        let untagged = matches!(tagged_format, TaggedFormat::Untagged);
        let partial = self.is_nested();

        let inner = match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                if self.args.null {
                    panic!("Only unit variants can be marked as `null`.");
                }

                let fields = fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let ty = &field.ty;

                        if partial {
                            quote! { SchemaType::infer_partial::<#ty>() }
                        } else {
                            quote! { SchemaType::infer::<#ty>() }
                        }
                    })
                    .collect::<Vec<_>>();

                if fields.len() == 1 {
                    let inner = &fields[0];

                    quote! { #inner }
                } else {
                    quote! {
                        SchemaType::tuple([
                            #(#fields),*
                        ])
                    }
                }
            }
            Fields::Unit => {
                if self.args.null || untagged {
                    quote! {
                        SchemaType::Null
                    }
                } else {
                    quote! {
                        SchemaType::literal(LiteralValue::String(#name.into()))
                    }
                }
            }
        };

        let outer = match tagged_format {
            TaggedFormat::Untagged => inner,
            TaggedFormat::External => {
                quote! {
                    SchemaType::structure([
                        SchemaField::new(#name, #inner),
                    ])
                }
            }
            TaggedFormat::Internal(tag) => {
                return quote! {
                    {
                        let mut schema = #inner;
                        schema.add_field(SchemaField::new(#tag, SchemaType::literal(LiteralValue::String(#name.into()))));
                        schema.set_partial(#partial);
                        schema
                    }
                };
            }
            TaggedFormat::Adjacent(tag, content) => {
                quote! {
                    SchemaType::structure([
                        SchemaField::new(#tag, SchemaType::literal(LiteralValue::String(#name.into()))),
                        SchemaField::new(#content, #inner),
                    ])
                }
            }
        };

        if partial {
            quote! {
                {
                    let mut schema = #outer;
                    schema.set_partial(#partial);
                    schema
                }
            }
        } else {
            outer
        }
    }
}

impl<'l> ToTokens for Variant<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;

        // Gather all attributes
        let mut attrs = vec![];

        if let Some(serde_meta) = self.get_serde_meta() {
            attrs.push(quote! { #[serde(#serde_meta)] });
        }

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }

        tokens.extend(match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let vis = &field.vis;
                        let ty = &field.ty;

                        if self.is_nested() {
                            quote! { #vis <#ty as schematic::Config>::Partial }
                        } else {
                            quote! { #vis #ty }
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #(#attrs)*
                    #name(#(#fields),*),
                }
            }
            Fields::Unit => quote! {
                #(#attrs)*
                #name,
            },
        });
    }
}
