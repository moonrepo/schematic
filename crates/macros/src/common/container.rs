use crate::common::{Field, Variant};
use crate::utils::map_option_quote;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Fields;

pub enum Container<'l> {
    NamedStruct { fields: Vec<Field<'l>> },
    Enum { variants: Vec<Variant<'l>> },
}

impl<'l> Container<'l> {
    pub fn has_nested(&self) -> bool {
        match self {
            Self::NamedStruct { fields, .. } => fields.iter().any(|v| v.is_nested()),
            Self::Enum { variants } => variants.iter().any(|v| v.is_nested()),
        }
    }

    pub fn generate_schema(&self, config_name: String, description: Option<String>) -> TokenStream {
        let description = if let Some(comment) = description {
            quote! {
                schema.description = Some(#comment.into());
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
                    let mut schema = StructType {
                        name: Some(#config_name.into()),
                        fields: vec![
                            #(#schema_types),*
                        ],
                        ..Default::default()
                    };

                    #description

                    SchemaType::Struct(schema)
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

                let default_index = map_option_quote("default_index", default_index);

                if is_all_unit_enum {
                    quote! {
                        let mut values = vec![];
                        let variants = vec![
                            #(#variants_types),*
                        ];

                        for variant in &variants {
                            if let SchemaType::Literal(lit) = &variant.type_of {
                                values.push(lit.value.clone().unwrap());
                            }
                        }

                        let mut schema = EnumType {
                            name: Some(#config_name.into()),
                            values,
                            variants: Some(variants),
                            #default_index
                            ..Default::default()
                        };

                        #description

                        SchemaType::Enum(schema)
                    }
                } else {
                    quote! {
                        let mut schema = UnionType {
                            name: Some(#config_name.into()),
                            variants_types: vec![
                                #(Box::new(#variants_types)),*
                            ],
                            #default_index
                            ..Default::default()
                        };

                        #description

                        SchemaType::Union(schema)
                    }
                }
            }
        }
    }
}
