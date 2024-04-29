use crate::common::FieldSerdeArgs;
use crate::utils::{extract_common_attrs, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Fields, Variant as NativeVariant};

#[derive(Clone)]
pub enum TaggedFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
    // Special case for unit only enums
    Unit,
}

// #[setting()], #[schema()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting, schema))]
pub struct VariantArgs {
    pub exclude: bool,
    pub null: bool,

    // config
    pub default: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    pub required: bool,
    pub validate: Option<Expr>,

    // serde
    pub rename: Option<String>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub casing_format: String,
    pub tagged_format: TaggedFormat,
    pub serde_args: FieldSerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l NativeVariant,
}

impl<'l> Variant<'l> {
    pub fn from(var: &NativeVariant) -> Variant {
        Variant {
            args: VariantArgs::from_attributes(&var.attrs).unwrap_or_default(),
            serde_args: FieldSerdeArgs::from_attributes(&var.attrs).unwrap_or_default(),
            attrs: extract_common_attrs(&var.attrs),
            casing_format: String::new(),
            tagged_format: TaggedFormat::Unit,
            name: &var.ident,
            value: var,
        }
    }

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn is_excluded(&self) -> bool {
        self.args.exclude
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn is_required(&self) -> bool {
        self.args.required
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

        let mut skipped = false;

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
            skipped = true;
        }

        if !skipped {
            if self.args.skip_serializing || self.serde_args.skip_serializing {
                meta.push(quote! { skip_serializing });
            }

            if self.args.skip_deserializing || self.serde_args.skip_deserializing {
                meta.push(quote! { skip_deserializing });
            }
        }

        if meta.is_empty() {
            return None;
        }

        Some(quote! {
            #(#meta),*
        })
    }

    pub fn generate_schema_type(&self, all_unit: bool) -> TokenStream {
        let name = self.get_name(Some(&self.casing_format));
        let tagged_format = if all_unit {
            &TaggedFormat::Unit
        } else {
            &self.tagged_format
        };

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
                            quote! { schema.infer_as_nested::<#ty>() }
                        } else {
                            quote! { schema.infer::<#ty>() }
                        }
                    })
                    .collect::<Vec<_>>();

                if fields.len() == 1 {
                    let inner = &fields[0];

                    quote! { #inner }
                } else {
                    quote! {
                        Schema::tuple(TupleType::new([
                            #(#fields),*
                        ]))
                    }
                }
            }
            Fields::Unit => {
                if self.args.null || untagged {
                    quote! {
                        Schema::null()
                    }
                } else {
                    quote! {
                        Schema::literal_value(LiteralValue::String(#name.into()))
                    }
                }
            }
        };

        let outer = match tagged_format {
            TaggedFormat::Unit => {
                quote! {
                    Schema {
                        name: Some(#name.into()),
                        ty: #inner.ty,
                        ..Default::default()
                    }
                }
            }
            TaggedFormat::Untagged => inner,
            TaggedFormat::External => {
                quote! {
                    Schema::structure(StructType::new([
                        (#name.into(), #inner),
                    ]))
                }
            }
            TaggedFormat::Internal(tag) => {
                return quote! {
                    {
                        let mut item = #inner;
                        item.add_field(
                            #tag,
                            Schema::literal_value(LiteralValue::String(#name.into())),
                        );
                        if #partial {
                            item.partialize();
                        }
                        item
                    }
                };
            }
            TaggedFormat::Adjacent(tag, content) => {
                quote! {
                    Schema::structure(StructType::new([
                        (#tag.into(), Schema::literal_value(LiteralValue::String(#name.into()))),
                        (#content.into(), #inner),
                    ]))
                }
            }
        };

        if partial {
            quote! {
                {
                    let mut item = #outer;
                    item
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
