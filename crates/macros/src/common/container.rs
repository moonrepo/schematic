use crate::common::{Field, TaggedFormat, Variant};
use proc_macro2::{Ident, TokenStream};
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

    pub fn generate_schema(
        &self,
        config_name: &Ident,
        description: Option<String>,
        casing_format: &str,
        tagged_format: TaggedFormat,
    ) -> TokenStream {
        let config_name = config_name.to_string();
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
                            Some(f.generate_schema_type(casing_format))
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

                let variants_types = variants
                    .iter()
                    .filter_map(|v| {
                        if v.is_excluded() {
                            None
                        } else {
                            Some(v.generate_schema_type(
                                casing_format,
                                if is_all_unit_enum {
                                    &TaggedFormat::Unit
                                } else {
                                    &tagged_format
                                },
                            ))
                        }
                    })
                    .collect::<Vec<_>>();

                if is_all_unit_enum {
                    quote! {
                        let mut values = vec![];
                        let variants = vec![
                            #(#variants_types),*
                        ];

                        for variant in &variants {
                            if let SchemaType::Literal(lit) = &variant.type_of {
                                values.push(lit.to_owned());
                            }
                        }

                        let mut schema = EnumType {
                            name: Some(#config_name.into()),
                            values,
                            variants: Some(variants),
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
