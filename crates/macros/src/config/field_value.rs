use crate::common::{FieldArgs, FieldValue, TypeInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Lit};

impl<'l> FieldValue<'l> {
    pub fn generate_default_value(&self, args: &FieldArgs) -> TokenStream {
        match self {
            Self::NestedList { .. } | Self::NestedMap { .. } => {
                quote! { Some(Default::default()) }
            }
            Self::NestedValue { info, .. } => {
                let partial_name = format_ident!("Partial{}", info.config.as_ref().unwrap());

                quote! { #partial_name::default_values(context)? }
            }
            Self::Value { value, .. } => {
                if let Some(expr) = args.default.as_ref() {
                    match expr {
                        Expr::Array(_) | Expr::Call(_) | Expr::Macro(_) | Expr::Tuple(_) => {
                            quote! { Some(#expr) }
                        }
                        Expr::Path(func) => {
                            quote! { handle_default_fn(#func(context))? }
                        }
                        Expr::Lit(lit) => match &lit.lit {
                            Lit::Str(string) => quote! {
                                Some(handle_default_fn(#value::try_from(#string))?)
                            },
                            other => quote! { Some(#other) },
                        },
                        invalid => {
                            let info = format!("{:?}", invalid);

                            panic!("Unsupported default value ({info}). May only provide literals, primitives, arrays, or tuples.");
                        }
                    }
                } else {
                    quote! { Some(Default::default()) }
                }
            }
        }
    }

    pub fn get_finalize_value(&self) -> Option<TokenStream> {
        match self {
            Self::NestedList { .. } | Self::NestedMap { .. } => {
                Some(self.map_data(quote! { value.finalize(context)? }))
            }
            Self::NestedValue { .. } => Some(self.map_data(quote! { data.finalize(context)? })),
            Self::Value { .. } => None,
        }
    }

    pub fn get_from_partial_value(&self) -> TokenStream {
        match self {
            Self::NestedList {
                item, item_info, ..
            } => self.map_data_with_info(
                quote! {
                    #item::from_partial(value)
                },
                item_info,
            ),
            Self::NestedMap {
                value, value_info, ..
            } => self.map_data_with_info(
                quote! {
                    #value::from_partial(value)
                },
                value_info,
            ),
            Self::NestedValue { info, .. } => {
                let config = info.config.as_ref();

                quote! {
                    #config::from_partial(data)
                }
            }
            Self::Value { .. } => quote! { data },
        }
    }

    pub fn get_merge_statement(&self, key: TokenStream, args: &FieldArgs) -> TokenStream {
        if let Self::NestedValue { .. } = self {
            if args.merge.is_some() {
                panic!("Nested configs do not support `merge` unless wrapped in a collection.");
            }

            return quote! {
                self.#key = merge_partial_setting(
                    self.#key.take(),
                    next.#key.take(),
                    context,
                )?;
            };
        };

        if let Some(func) = args.merge.as_ref() {
            quote! {
                self.#key = merge_setting(
                    self.#key.take(),
                    next.#key.take(),
                    context,
                    #func,
                )?;
            }
        } else {
            quote! {
                if next.#key.is_some() {
                    self.#key = next.#key;
                }
            }
        }
    }

    pub fn get_validate_statement(&self, key: &str) -> Option<TokenStream> {
        match self {
            Self::NestedList { .. } => Some(quote! {
                for (i, item) in setting.iter().enumerate() {
                    if let Err(nested_error) = item.validate_with_path(context, finalize, path.join_key(#key).join_index(i)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }
            }),
            Self::NestedMap { .. } => Some(quote! {
                for (key, value) in setting {
                    if let Err(nested_error) = value.validate_with_path(context, finalize, path.join_key(#key).join_key(key)) {
                        errors.push(schematic::ValidateErrorType::nested(nested_error));
                    }
                }
            }),
            Self::NestedValue { .. } => Some(quote! {
                if let Err(nested_error) = setting.validate_with_path(context, finalize, path.join_key(#key)) {
                    errors.push(schematic::ValidateErrorType::nested(nested_error));
                }
            }),
            Self::Value { .. } => {
                // Handled in parent struct
                None
            }
        }
    }

    pub fn map_data(&self, mapped_data: TokenStream) -> TokenStream {
        match self {
            Self::NestedList { collection, .. } => {
                quote! {
                    {
                        let mut result = #collection::default();
                        for value in data {
                            result.push(#mapped_data);
                        }
                        result
                    }
                }
            }
            Self::NestedMap { collection, .. } => {
                quote! {
                    {
                        let mut result = #collection::default();
                        for (key, value) in data {
                            result.insert(key, #mapped_data);
                        }
                        result
                    }
                }
            }
            Self::NestedValue { .. } | Self::Value { .. } => {
                quote! { #mapped_data }
            }
        }
    }

    pub fn map_data_with_info(&self, mapped_data: TokenStream, info: &TypeInfo) -> TokenStream {
        let mut data = mapped_data;

        if info.boxed {
            data = quote! { Box::new(#data) };
        }

        if info.optional {
            data = quote! { Some(#data) };
        }

        self.map_data(data)
    }
}
