use crate::common::{Field, Variant};
use crate::utils::{extract_comment, extract_deprecated, map_option_argument_quote};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Fields};

pub enum Container<'l> {
    NamedStruct { fields: Vec<Field<'l>> },
    UnnamedStruct { fields: Vec<Field<'l>> },
    Enum { variants: Vec<Variant<'l>> },
}

impl Container<'_> {
    pub fn has_nested(&self) -> bool {
        match self {
            Self::NamedStruct { fields } => fields.iter().any(|v| v.is_nested()),
            Self::UnnamedStruct { fields } => fields.iter().any(|v| v.is_nested()),
            Self::Enum { variants } => variants.iter().any(|v| v.is_nested()),
        }
    }

    pub fn generate_settings_metadata(&self) -> TokenStream {
        let mut settings = vec![];

        match self {
            Self::NamedStruct { fields } => {
                for field in fields {
                    let name = field.get_name(Some(&field.casing_format));
                    let env_key = if let Some(value) = field.get_env_var() {
                        quote!(Some(#value.into()))
                    } else {
                        quote!(None)
                    };
                    let type_alias = format!("{}", field.value.to_token_stream());

                    settings.push(quote! {
                        (#name.into(), schematic::ConfigSetting {
                            env_key: #env_key,
                            type_alias: #type_alias.into(),
                        }),
                    });
                }
            }
            Self::UnnamedStruct { fields } => {
                for field in fields {
                    let name = field.index.to_string();
                    let env_key = if let Some(value) = field.get_env_var() {
                        quote!(Some(#value.into()))
                    } else {
                        quote!(None)
                    };
                    let type_alias = format!("{}", field.value.to_token_stream());

                    settings.push(quote! {
                        (#name.into(), schematic::ConfigSetting {
                            env_key: #env_key,
                            type_alias: #type_alias.into(),
                        }),
                    });
                }
            }
            Self::Enum { variants } => {
                for variant in variants {
                    let name = variant.get_name(Some(&variant.casing_format));
                    let type_alias = format!("{}", variant.value.to_token_stream());

                    settings.push(quote! {
                        (#name.into(), schematic::ConfigSetting {
                            env_key: None,
                            type_alias: #type_alias.into(),
                        }),
                    });
                }
            }
        };

        quote! {
            std::collections::HashMap::from_iter([
                #(#settings)*
            ])
        }
    }

    pub fn generate_schema(&self, attrs: &[&Attribute]) -> TokenStream {
        let deprecated = if let Some(comment) = extract_deprecated(attrs) {
            quote! { schema.set_deprecated(#comment); }
        } else {
            quote! {}
        };
        let description = if let Some(comment) = extract_comment(attrs) {
            quote! { schema.set_description(#comment); }
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
                            Some(f.generate_schema_type(true))
                        }
                    })
                    .collect::<Vec<_>>();

                if fields.is_empty() {
                    quote! {
                        #deprecated
                        #description
                        schema.structure(StructType::default())
                    }
                } else {
                    quote! {
                        #deprecated
                        #description
                        schema.structure(StructType::new([
                            #(#schema_types),*
                        ]))
                    }
                }
            }
            Self::UnnamedStruct { fields, .. } => {
                let schema_types = fields
                    .iter()
                    .filter_map(|f| {
                        if f.is_excluded() {
                            None
                        } else {
                            Some(f.generate_schema_type(false))
                        }
                    })
                    .collect::<Vec<_>>();

                if fields.len() == 1 {
                    let single_type = &schema_types[0];

                    quote! {
                        let mut schema = #single_type;
                        #deprecated
                        #description
                        schema
                    }
                } else {
                    quote! {
                        #deprecated
                        #description
                        schema.tuple(TupleType::new([
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
                        #deprecated
                        #description
                        schema.enumerable(EnumType::from_schemas(
                            [
                                #(#variants_types),*
                            ],
                            #default_index,
                        ))
                    }
                } else {
                    quote! {
                        #deprecated
                        #description
                        schema.union(UnionType::from_schemas(
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
