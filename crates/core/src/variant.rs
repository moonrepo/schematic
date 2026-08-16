use crate::args::{
    NestedArg, PartialArg, SerdeContainerArgs, SerdeFieldArgs, SerdeIoDirection, SerdeRenameArg,
};
use crate::container::ContainerArgs;
use crate::utils::{ImplResult, is_inheritable_attribute};
use crate::variant_value::VariantValue;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::rc::Rc;
use syn::{Attribute, ExprPath, Fields, FieldsUnnamed, Ident, Index, Variant as NativeVariant};

// #[setting()], #[schema()]
#[derive(Debug, Default, FromAttributes)]
#[darling(default, attributes(setting, schema))]
pub struct VariantArgs {
    pub default: bool,
    #[cfg(feature = "schema")]
    pub exclude: bool,
    pub merge: Option<ExprPath>,
    pub nested: Option<NestedArg>,
    pub null: bool,
    pub partial: Option<PartialArg>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    #[cfg(feature = "validate")]
    pub validate: Option<crate::args::ValidateArg>,

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
    pub values: Vec<VariantValue>,

    // args
    pub args: VariantArgs,
    pub container_args: Rc<ContainerArgs>,
    pub serde_args: SerdeFieldArgs,
    pub serde_container_args: Rc<SerdeContainerArgs>,

    // inherited
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub fields: Fields,
}

impl Variant {
    pub fn new(
        variant: NativeVariant,
        container_args: Rc<ContainerArgs>,
        serde_container_args: Rc<SerdeContainerArgs>,
    ) -> Variant {
        let args = VariantArgs::from_attributes(&variant.attrs).unwrap();
        let serde_args = SerdeFieldArgs::from_attributes(&variant.attrs).unwrap();

        let variant = Self {
            attrs: variant.attrs,
            container_args,
            ident: variant.ident,
            serde_args,
            serde_container_args,
            values: match &variant.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|field| VariantValue::new(field.ty.clone(), args.nested.as_ref()))
                    .collect(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .map(|field| VariantValue::new(field.ty.clone(), args.nested.as_ref()))
                    .collect(),
                Fields::Unit => vec![],
            },
            fields: variant.fields,
            args,
        };
        variant.validate_args();
        variant
    }

    fn validate_args(&self) {
        if self.is_required()
            && self
                .values
                .iter()
                .any(|value| !value.is_outer_option_wrapped())
        {
            panic!("Cannot use `required` with non-optional settings.");
        }

        #[allow(clippy::collapsible_else_if)]
        if self.is_unit_variant() {
            if self.args.merge.is_some() {
                panic!("Cannot use `merge` with unit variants.");
            }

            if self.args.nested.is_some() {
                panic!("Cannot use `nested` with unit variants.");
            }

            if self.args.required {
                panic!("Cannot use `required` with unit variants.");
            }

            #[cfg(feature = "validate")]
            if self.args.validate.is_some() {
                panic!("Cannot use `validate` with unit variants.");
            }
        } else {
            if self.args.null {
                panic!("Can only use `null` with unit variants.");
            }
        }
    }

    pub fn get_name(&self) -> String {
        let dir = SerdeIoDirection::From;

        if let Some(name) = self.args.rename.as_ref().and_then(|rn| rn.get_name(dir)) {
            return name.into();
        }

        if let Some(name) = self
            .serde_args
            .rename
            .as_ref()
            .and_then(|rn| rn.get_name(dir))
        {
            return name.into();
        }

        self.ident.to_string()
    }

    pub fn is_default(&self) -> bool {
        self.args.default
    }

    pub fn is_excluded(&self) -> bool {
        #[cfg(feature = "schema")]
        {
            self.args.exclude
        }

        #[cfg(not(feature = "schema"))]
        {
            false
        }
    }

    pub fn is_nested(&self) -> bool {
        self.args
            .nested
            .as_ref()
            .is_some_and(|nested| nested.is_nested())
    }

    pub fn is_required(&self) -> bool {
        self.args.required
    }

    pub fn is_unit_variant(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get_partial_attributes(&self) -> Vec<TokenStream> {
        let mut attrs = vec![];

        // Serde attributes come first, so that they take precedence
        // over any provided by the user via `partial(serde(...))`
        let serde_args = self.get_partial_serde_attribute_args();

        if !serde_args.is_empty() {
            attrs.push(quote! { #[serde(#serde_args)] });
        }

        // Inherit non-schematic attributes from the variant,
        // like `doc`, `allow`, and `deprecated`
        for attr in &self.attrs {
            if is_inheritable_attribute(attr) {
                attrs.push(quote! { #attr });
            }
        }

        // Then apply any user-provided partial attributes
        if let Some(partial) = &self.args.partial {
            attrs.extend(partial.get_attributes());
        }

        attrs
    }

    pub fn get_partial_serde_attribute_args(&self) -> TokenStream {
        let mut meta = vec![];

        // Aliases can be provided by both, so combine them
        let mut aliases: Vec<&String> = vec![];

        for alias in self.args.alias.iter().chain(self.serde_args.alias.iter()) {
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }

        for alias in aliases {
            meta.push(quote! { alias = #alias });
        }

        // Setting attributes take precedence over serde attributes
        if let Some(rename) = self
            .args
            .rename
            .as_ref()
            .or(self.serde_args.rename.as_ref())
            .filter(|rename| !rename.is_empty())
        {
            meta.push(rename.get_meta("rename"));
        }

        if self.args.skip || self.serde_args.skip {
            meta.push(quote! { skip });
        } else {
            if self.args.skip_serializing || self.serde_args.skip_serializing {
                meta.push(quote! { skip_serializing });
            }

            if self.args.skip_deserializing || self.serde_args.skip_deserializing {
                meta.push(quote! { skip_deserializing });
            }
        }

        if self.args.untagged || self.serde_args.untagged {
            meta.push(quote! { untagged });
        }

        if self.serde_args.other {
            meta.push(quote! { other });
        }

        quote! {
            #(#meta),*
        }
    }

    pub fn impl_full_from_partial(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(fields) => {
                self.map_unnamed_match_custom(name, partial_name, fields, |outer_names, _| {
                    let items = outer_names
                        .iter()
                        .enumerate()
                        .map(|(index, o)| {
                            if self.values[index].requires_from_partial_mapping() {
                                self.values[index].impl_full_from_partial_value(o).value
                            } else {
                                quote! { #o }
                            }
                        })
                        .collect::<Vec<_>>();

                    quote! {
                        Self::#name(#(#items),*)
                    }
                })
            }
            Fields::Unit => quote! {
                #partial_name::#name => Self::#name,
            },
        };

        res
    }

    pub fn impl_partial_default_value(&self) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;

        res.value = match &self.fields {
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

    /// Generate a deserialization attempt for this variant, for use within
    /// untagged enums, where each variant is tried in order.
    pub fn impl_partial_type_deserialize(&self, partial_name: &Ident) -> ImplResult {
        let mut res = ImplResult::default();
        let name = &self.ident;
        let name_string = self.get_name();

        let deserializer = quote! {
            schematic::serde_content::Deserializer::new(content.clone())
                .coerce_numbers()
                .human_readable()
        };

        res.value = match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(_) => {
                let types = self
                    .values
                    .iter()
                    .map(|value| value.get_partial_type())
                    .collect::<Vec<_>>();

                if types.len() == 1 {
                    let ty = &types[0];

                    quote! {
                        match <#ty as serde::Deserialize>::deserialize(#deserializer) {
                            Ok(value) => return Ok(#partial_name::#name(value)),
                            Err(error) => errors.push((#name_string, error.to_string())),
                        }
                    }
                } else {
                    // Deserialize multiple values as a tuple, then destructure
                    let indexes = (0..types.len()).map(Index::from).collect::<Vec<_>>();

                    quote! {
                        match <(#(#types),*) as serde::Deserialize>::deserialize(#deserializer) {
                            Ok(value) => return Ok(#partial_name::#name(#(value.#indexes),*)),
                            Err(error) => errors.push((#name_string, error.to_string())),
                        }
                    }
                }
            }
            // Unit variants in untagged enums are represented as null
            Fields::Unit => quote! {
                match <() as serde::Deserialize>::deserialize(#deserializer) {
                    Ok(_) => return Ok(#partial_name::#name),
                    Err(error) => errors.push((#name_string, error.to_string())),
                }
            },
        };

        res
    }

    pub fn impl_partial_finalize(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.fields {
            Fields::Named(_) | Fields::Unit => {
                res.no_value = true;
            }
            Fields::Unnamed(fields) => {
                if !self.is_nested() && self.args.transform.is_none() {
                    res.no_value = true;
                } else {
                    let name = &self.ident;

                    res.value = self.map_unnamed_match(name, fields, |outer_names, _| {
                        let items = outer_names
                            .iter()
                            .enumerate()
                            .map(|(i, o)| {
                                let value = if self.is_nested() {
                                    // Variant values have no partial `Option`
                                    // wrapper, so walk all layers
                                    self.values[i].impl_partial_finalize_nested(o, false).value
                                } else {
                                    quote! { #o }
                                };

                                if let Some(func) = &self.args.transform {
                                    quote! { #func(#value, context)? }
                                } else {
                                    value
                                }
                            })
                            .collect::<Vec<_>>();

                        quote! {
                            Self::#name(#(#items),*)
                        }
                    });
                }
            }
        }

        res
    }

    pub fn impl_partial_merge(&self) -> ImplResult {
        let mut res = ImplResult::default();

        match &self.fields {
            Fields::Named(_) => {
                res.no_value = true;
            }
            Fields::Unnamed(fields) => {
                let name = &self.ident;

                match &self.args.merge {
                    Some(func) => {
                        if self.is_nested()
                            && self
                                .values
                                .first()
                                .is_none_or(|value| !value.is_collection())
                        {
                            panic!(
                                "Nested configs do not support `merge` unless wrapped in a collection."
                            );
                        }

                        res.value = self.map_unnamed_match(&self.ident, fields, |outer_names, inner_names| {
                            if outer_names.len() == 1 {
                                quote! {
                                    if let Self::#name(na) = next {
                                        *self = Self::#name(
                                            #func(pa.to_owned(), na, context)?.unwrap_or_default(),
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
                    }
                    None => {
                        // Nested configs are merged recursively, but collections
                        // of them are replaced, as there's no way to know how to
                        // pair up their items. Define `merge` to customize this.
                        let mergeable = self.is_nested()
                            && self.values.iter().any(|value| !value.is_collection());

                        if mergeable {
                            let mut requires_internal = false;

                            res.value = self.map_unnamed_match(
                                &self.ident,
                                fields,
                                |outer_names, inner_names| {
                                    let statements = outer_names
                                        .iter()
                                        .enumerate()
                                        .map(|(index, o)| {
                                            let i = &inner_names[index];
                                            let value = &self.values[index];

                                            // Collections are replaced in place
                                            if value.is_collection() {
                                                return quote! { *#o = #i; };
                                            }

                                            // Variant values are not wrapped by the
                                            // partial, so all layers must be handled
                                            let merge = value.impl_partial_merge_nested(
                                                &quote! { #o },
                                                &quote! { #i },
                                                false,
                                            );

                                            // Wrap with the manager when the value is optional
                                            if merge.requires_internal {
                                                requires_internal = true;

                                                let inner = merge.value;

                                                quote! { MergeManager::new(context)#inner; }
                                            } else {
                                                merge.value
                                            }
                                        })
                                        .collect::<Vec<_>>();

                                    quote! {
                                        if let Self::#name(#(#inner_names),*) = next {
                                            #(#statements)*
                                        } else {
                                            *self = next;
                                        }
                                    }
                                },
                            );

                            res.requires_internal = requires_internal;
                        } else {
                            res.no_value = true;
                        }
                    }
                };
            }
            Fields::Unit => {
                res.no_value = true;
            }
        };

        res
    }

    pub fn impl_partial_validate(&self) -> ImplResult {
        let Fields::Unnamed(fields) = &self.fields else {
            return ImplResult::skipped();
        };

        let value = self.map_unnamed_match(&self.ident, fields, |outer_names, _| {
            let mut statements = vec![];
            let name_string = self.ident.to_string();

            #[cfg(feature = "validate")]
            if let Some(expr) = self.args.validate.as_deref() {
                use syn::Expr;

                let func = match expr {
                    // func(arg)() - already returns a boxed validator
                    Expr::Call(func) => quote! { #func },
                    // func() - must be boxed
                    Expr::Path(func) => quote! { Box::new(#func) },
                    _ => {
                        panic!("Unsupported `validate` syntax.");
                    }
                };

                statements.push(quote! {
                    validate.check(#name_string, (#(#outer_names),*), self, #func);
                });
            }

            if self.is_required() {
                statements.push(quote! {
                    if [#(#outer_names),*].iter().any(|v| v.is_none()) {
                        validate.required(#name_string);
                    }
                });
            }

            if self.is_nested() {
                statements.extend(
                    outer_names
                        .iter()
                        .enumerate()
                        .map(|(index, o)| {
                            let name_index = format!("{name_string}.{index}");

                            // Variant values are not wrapped by the partial,
                            // so all layers must be handled
                            self.values[index]
                                .impl_partial_validate_nested(&name_index, o, false)
                                .value
                        })
                        .collect::<Vec<_>>(),
                );
            }

            quote! {
                #(#statements)*
            }
        });

        ImplResult {
            value,
            ..Default::default()
        }
    }

    fn map_unnamed_match<F>(&self, name: &Ident, fields: &FieldsUnnamed, factory: F) -> TokenStream
    where
        F: FnOnce(&[Ident], &[Ident]) -> TokenStream,
    {
        let self_name = format_ident!("Self");

        self.map_unnamed_match_custom(name, &self_name, fields, factory)
    }
}

// Only used for partials!
impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = self.get_partial_attributes();
        let name = &self.ident;

        tokens.extend(match &self.fields {
            Fields::Named(_) => panic!("Enums with named fields are not supported!"),
            Fields::Unnamed(_) => {
                let types = self
                    .values
                    .iter()
                    .map(|value| value.get_partial_type())
                    .collect::<Vec<_>>();

                quote! {
                    #(#attrs)*
                    #name(#(#types),*),
                }
            }
            Fields::Unit => quote! {
                #(#attrs)*
                #name,
            },
        });
    }
}

impl Variant {
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
            let outer_name = format_ident!("p{}", count as char);
            let inner_name = format_ident!("n{}", count as char);

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
