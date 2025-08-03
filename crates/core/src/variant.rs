use crate::args::{NestedArg, SerdeContainerArgs, SerdeFieldArgs, SerdeRenameArg};
use crate::container::ContainerArgs;
use crate::utils::ImplResult;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, ExprPath, Fields, FieldsUnnamed, Ident, Variant as NativeVariant};

// #[setting()], #[schema()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting, schema))]
pub struct VariantArgs {
    pub default: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<NestedArg>,

    // serde
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub untagged: bool,
}

#[derive(Debug)]
pub struct Variant {
    // args
    pub args: VariantArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub value: Fields,
}

impl Variant {
    pub fn new(
        variant: NativeVariant,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Variant {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap();

        Variant {
            args,
            attrs: variant.attrs,
            container_args,
            ident: variant.ident,
            serde_args,
            serde_container_args,
            value: variant.fields,
        }
    }

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn is_nested(&self) -> bool {
        self.args
            .nested
            .as_ref()
            .is_some_and(|nested| nested.is_nested())
    }

    pub fn impl_partial_default_value(&self) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.value {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
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
        };

        res
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.value {
            Fields::Named(_) => {
                res.no_value = true;
            }
            Fields::Unnamed(fields) => {
                let name = &self.ident;

                if self.is_nested() {
                    if self.args.merge.is_some() {
                        panic!("Nested variants do not support `merge`.");
                    }

                    res.value =
                        self.map_unnamed_match(&self.ident, fields, |outer_names, inner_names| {
                            let statements = outer_names
                                .iter()
                                .enumerate()
                                .map(|(index, o)| {
                                    let i = &inner_names[index];
                                    quote! { #o.merge(context, #i)?; }
                                })
                                .collect::<Vec<_>>();

                            quote! {
                                if let Self::#name(#(#inner_names),*) = next {
                                    #(#statements)*
                                } else {
                                    *self = next;
                                }
                            }
                        });
                } else if let Some(func) = &self.args.merge {
                    res.value =
                        self.map_unnamed_match(&self.ident, fields, |outer_names, inner_names| {
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
                        });
                } else {
                    res.no_value = true;
                }
            }
            Fields::Unit => {
                res.no_value = true;
            }
        };

        res
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

        for _ in &fields.unnamed {
            let outer_name = format_ident!("{}o", count as char);
            let inner_name = format_ident!("{}i", count as char);

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
