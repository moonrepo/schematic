use crate::common::Variant;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Expr, Fields, FieldsUnnamed};

impl<'l> Variant<'l> {
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
                    if let Err(error) = #func(#value, self, context, finalize) {
                        errors.push(error);
                    }
                });
            }

            if self.is_required() {
                let name = self.get_name(Some(&self.casing_format));

                stmts.push(quote! {
                    if finalize && [#(#outer_names),*].iter().any(|v| v.is_none()) {
                        let mut error = schematic::ValidateError::required();
                        error.prepend_path(path.join_key(#name));
                        errors.push(error);
                    }
                });
            }

            if self.is_nested() {
                stmts.extend(outer_names
                    .iter()
                    .enumerate()
                    .map(|(index, o)| {
                        quote! {
                            if let Err(nested_errors) = #o.validate_with_path(context, finalize, path.join_index(#index)) {
                                errors.extend(nested_errors);
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
