use crate::common::{Field, Variant};
use crate::utils::map_option_argument_quote;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Fields;

pub enum Container<'l> {
    NamedStruct { fields: Vec<Field<'l>> },
    UnnamedStruct { fields: Vec<Field<'l>> },
    Enum { variants: Vec<Variant<'l>> },
}

impl<'l> Container<'l> {
    pub fn has_nested(&self) -> bool {
        match self {
            Self::NamedStruct { fields, .. } => fields.iter().any(|v| v.is_nested()),
            Self::UnnamedStruct { fields, .. } => fields.iter().any(|v| v.is_nested()),
            Self::Enum { variants } => variants.iter().any(|v| v.is_nested()),
        }
    }

    pub fn generate_schema(&self, description: Option<String>) -> TokenStream {
        let description = if let Some(comment) = description {
            quote! {
                schema.set_description(#comment);
            }
        } else {
            quote! {}
        };

        match self {
            Self::NamedStruct { fields, .. } => {
                let schema_types = fields
                    .iter()
                    .filter_map(|f| {
                        if f.is_excluded() {
                            None
                        } else {
                            Some(f.generate_schema_type())
                        }
                    })
                    .collect::<Vec<_>>();

                quote! {
                    #description
                    schema.structure(StructType::new([
                        #(#schema_types),*
                    ]))
                }
            }
            Self::UnnamedStruct { fields, .. } => {
                let schema_types = fields
                    .iter()
                    .filter_map(|f| {
                        if f.is_excluded() {
                            None
                        } else {
                            Some(f.generate_schema_type())
                        }
                    })
                    .collect::<Vec<_>>();

                if fields.len() == 1 {
                    let single_type = &schema_types[0];

                    quote! {
                        let mut schema = #single_type;
                        #description
                        schema
                    }
                } else {
                    quote! {
                        #description
                        schema.array(ArrayType::new([
                            #(#schema_types),*
                        ]))
                    }
                }
            }
            Self::Enum { variants } => {
                let is_all_unit_enum = variants
                    .iter()
                    .all(|v| matches!(v.value.fields, Fields::Unit));
                let mut default_index = None;

                let variants_types = variants
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| {
                        if v.is_default() {
                            default_index = Some(i);
                        }

                        if v.is_excluded() {
                            None
                        } else {
                            Some(v.generate_schema_type(is_all_unit_enum))
                        }
                    })
                    .collect::<Vec<_>>();

                let default_index = map_option_argument_quote(default_index);

                if is_all_unit_enum {
                    quote! {
                        #description
                        schema.enumerable(EnumType::from_macro(
                            [
                                #(#variants_types),*
                            ],
                            #default_index,
                        ))
                    }
                } else {
                    quote! {
                        #description
                        schema.union(UnionType::from_macro(
                            [
                                #(#variants_types),*
                            ],
                            #default_index,
                        ))
                    }
                }
            }
        }
    }
}
