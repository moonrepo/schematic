use crate::utils::{extract_common_attrs, format_case};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Fields, FieldsUnnamed, Variant as NativeVariant};

pub enum TaggedFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
}

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeArgs {
    pub alias: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
}

// #[variant()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(setting))]
pub struct VariantArgs {
    pub default: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    pub null: bool,
    pub validate: Option<Expr>,

    // serde
    pub rename: Option<String>,
    pub skip: bool,
}

pub struct Variant<'l> {
    pub args: VariantArgs,
    pub serde_args: SerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l NativeVariant,
}

impl<'l> Variant<'l> {
    pub fn from(var: &NativeVariant) -> Variant {
        Variant {
            args: VariantArgs::from_attributes(&var.attrs).unwrap_or_default(),
            serde_args: SerdeArgs::from_attributes(&var.attrs).unwrap_or_default(),
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

    pub fn generate_default_value(&self) -> TokenStream {
        let name = &self.name;

        match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|_| {
                        quote! { Default::default() }
                    })
                    .collect::<Vec<_>>();

                quote! { #name(#(#fields),*) }
            }
            Fields::Unit => quote! { #name },
        }
    }

    pub fn generate_finalize_statement(&self) -> Option<TokenStream> {
        let name = &self.name;

        match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                if !self.is_nested() {
                    return None;
                }

                Some(self.map_unnamed_match(self.name, fields, |outer_names, _| {
                    let stmts = outer_names
                        .iter()
                        .map(|o| {
                            quote! { #o.finalize(context)? }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        Self::#name(#(#stmts),*)
                    }
                }))
            }
            Fields::Unit => None,
        }
    }

    pub fn generate_merge_statement(&self) -> Option<TokenStream> {
        let name = &self.name;
        let args = &self.args;

        match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                if self.is_nested() {
                    if args.merge.is_some() {
                        panic!("Nested variants do not support `merge`.");
                    }

                    return Some(self.map_unnamed_match(
                        self.name,
                        fields,
                        |outer_names, inner_names| {
                            let merge_stmts = outer_names
                                .iter()
                                .enumerate()
                                .map(|(index, o)| {
                                    let i = &inner_names[index];
                                    quote! { #o.merge(context, #i)?; }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                if let Self::#name(#(#inner_names),*) = next {
                                    #(#merge_stmts)*
                                } else {
                                    *self = next;
                                }
                            }
                        },
                    ));
                }

                if let Some(func) = args.merge.as_ref() {
                    return Some(self.map_unnamed_match(
                        self.name,
                        fields,
                        |outer_names, inner_names| {
                            if outer_names.len() == 1 {
                                quote! {
                                    if let Self::#name(ai) = next {
                                        *self = Self::#name(
                                            #func(ao.to_owned(), ai, context)?.unwrap_or_default(),
                                        );
                                    } else {
                                        *self = next;
                                    }
                                }
                            } else {
                                let defaults = outer_names
                                    .iter()
                                    .map(|_| {
                                        quote! { Default::default() }
                                    })
                                    .collect::<Vec<_>>();

                                quote! {
                                    if let Self::#name(#(#inner_names),*) = next {
                                        if let Some((#(#outer_names),*)) = #func(
                                            (#(#outer_names.to_owned()),*),
                                            (#(#inner_names),*),
                                            context,
                                        )? {
                                            *self = Self::#name(#(#outer_names),*);
                                        } else {
                                            *self = Self::#name(#(#defaults),*);
                                        }
                                    } else {
                                        *self = next;
                                    }
                                }
                            }
                        },
                    ));
                }

                None
            }
            Fields::Unit => {
                if args.merge.is_some() {
                    panic!("Unit variants do not support `merge`.")
                }

                None
            }
        }
    }

    pub fn generate_validate_statement(&self) -> Option<TokenStream> {
        let Fields::Unnamed(fields) = &self.value.fields else {
            return None;
        };

        Some(self.map_unnamed_match(self.name, fields, |outer_names, _| {
            let mut stmts = vec![];

            if let Some(expr) = self.args.validate.as_ref() {
                let func = match expr {
                    // func(arg)()
                    Expr::Call(func) => quote! { #func },
                    // func()
                    Expr::Path(func) => quote! { #func },
                    _ => {
                        panic!("Unsupported `validate` syntax.");
                    }
                };

                let value = if outer_names.len() == 1 {
                    quote! { #(#outer_names),* }
                } else {
                    quote! { (#(#outer_names),*) }
                };

                stmts.push(quote! {
                    if let Err(error) = #func(#value, self, context) {
                        errors.push(schematic::ValidateErrorType::setting(
                            path.clone(),
                            error,
                        ));
                    }
                });
            }

            if self.is_nested() {
                stmts.extend(outer_names
                    .iter()
                    .enumerate()
                    .map(|(index, o)| {
                        quote! {
                            if let Err(nested_error) = #o.validate_with_path(context, path.join_index(#index)) {
                                errors.push(schematic::ValidateErrorType::nested(nested_error));
                            }
                        }
                    })
                    .collect::<Vec<_>>());
            }

            quote! {
                #(#stmts)*
            }
        }))
    }

    pub fn generate_from_partial_value(&self, partial_name: &Ident) -> TokenStream {
        let name = &self.name;

        match &self.value.fields {
            Fields::Named(_) => unreachable!(),
            Fields::Unnamed(fields) => {
                self.map_unnamed_match_custom(self.name, partial_name, fields, |outer_names, _| {
                    let stmts = outer_names
                        .iter()
                        .enumerate()
                        .map(|(index, o)| {
                            if self.is_nested() {
                                let ty = &fields.unnamed[index].ty;

                                quote! { #ty::from_partial(#o) }
                            } else {
                                quote! { #o }
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        Self::#name(#(#stmts),*)
                    }
                })
            }
            Fields::Unit => {
                quote! {
                    #partial_name::#name => Self::#name,
                }
            }
        }
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

    fn map_unnamed_match<F>(&self, name: &Ident, fields: &FieldsUnnamed, factory: F) -> TokenStream
    where
        F: FnOnce(&[Ident], &[Ident]) -> TokenStream,
    {
        let self_name = format_ident!("Self");

        self.map_unnamed_match_custom(name, &self_name, fields, factory)
    }

    fn map_unnamed_match_custom<F>(
        &self,
        name: &Ident,
        self_name: &Ident,
        fields: &FieldsUnnamed,
        factory: F,
    ) -> TokenStream
    where
        F: FnOnce(&[Ident], &[Ident]) -> TokenStream,
    {
        let mut count: u8 = 97; // a
        let mut outer_names = vec![];
        let mut inner_names = vec![];
        let mut merge_stmts = vec![];

        for _ in &fields.unnamed {
            let outer_name = format_ident!("{}o", count as char);
            let inner_name = format_ident!("{}i", count as char);

            merge_stmts.push(quote! {
                #outer_name.merge(context, #inner_name)?;
            });

            outer_names.push(outer_name);
            inner_names.push(inner_name);

            count += 1;
        }

        let inner = factory(&outer_names, &inner_names);

        quote! {
            #self_name::#name(#(#outer_names),*) => {
                #inner
            },
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
