use crate::common::FieldValue;
use crate::utils::{
    extract_comment, extract_common_attrs, extract_deprecated, format_case, preserve_str_literal,
};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Expr, ExprPath, Field as NativeField, Lit, Type};

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct FieldSerdeArgs {
    pub alias: Option<String>,
    pub flatten: bool,
    pub rename: Option<String>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
}

// #[schema()], #[setting()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {
    // schema
    pub exclude: bool,

    // config
    #[darling(with = "preserve_str_literal", map = "Some")]
    pub default: Option<Expr>,
    pub env: Option<String>,
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    pub parse_env: Option<ExprPath>,
    pub validate: Option<Expr>,

    // serde
    pub flatten: bool,
    pub rename: Option<String>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
}

pub struct Field<'l> {
    pub args: FieldArgs,
    pub serde_args: FieldSerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub value: &'l Type,
    pub value_type: FieldValue<'l>,
}

impl<'l> Field<'l> {
    pub fn from(field: &NativeField) -> Field {
        let args = FieldArgs::from_attributes(&field.attrs).unwrap_or_default();
        let serde_args = FieldSerdeArgs::from_attributes(&field.attrs).unwrap_or_default();

        let field = Field {
            name: field.ident.as_ref().unwrap(),
            attrs: extract_common_attrs(&field.attrs),
            value: &field.ty,
            value_type: if args.nested {
                FieldValue::nested(&field.ty)
            } else {
                FieldValue::value(&field.ty)
            },
            args,
            serde_args,
        };

        if field.args.default.is_some() {
            if field.is_nested() {
                panic!("Cannot use defaults with `nested` configs.");
            }

            if field.is_optional() {
                panic!("Cannot use defaults with optional settings.");
            }
        }

        field
    }

    pub fn is_excluded(&self) -> bool {
        self.args.exclude
    }

    pub fn is_extendable(&self) -> bool {
        self.args.extend
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn is_optional(&self) -> bool {
        self.value_type.is_optional()
    }

    pub fn is_skipped(&self) -> bool {
        self.args.skip || self.serde_args.skip
    }

    pub fn get_name(&self, casing_format: Option<&str>) -> String {
        if let Some(local) = &self.args.rename {
            local.to_owned()
        } else if let Some(serde) = &self.serde_args.rename {
            serde.to_owned()
        } else if let Some(format) = casing_format {
            format_case(format, &self.name.to_string(), false)
        } else {
            self.name.to_string()
        }
    }

    pub fn get_serde_meta(&self) -> Option<TokenStream> {
        let mut meta = vec![];

        if self.args.flatten || self.serde_args.flatten {
            meta.push(quote! { flatten });
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
            } else {
                meta.push(quote! { skip_serializing_if = "Option::is_none" });
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

    pub fn generate_schema_type(&self, casing_format: &str) -> TokenStream {
        let name = self.get_name(Some(casing_format));
        let value = self.value;

        let hidden = if self.is_skipped() {
            quote! { hidden: true, }
        } else {
            quote! {}
        };
        let nullable = if self.is_optional() {
            quote! { nullable: true, }
        } else {
            quote! {}
        };
        let description = if let Some(comment) = extract_comment(&self.attrs) {
            quote! {
                description: Some(#comment.into()),
            }
        } else {
            quote! {}
        };
        let deprecated = if let Some(deprecated) = extract_deprecated(&self.attrs) {
            quote! {
                deprecated: Some(#deprecated.into()),
            }
        } else {
            quote! {}
        };
        let env_var = if let Some(var) = &self.args.env {
            quote! {
                env_var: Some(#var.into()),
            }
        } else {
            quote! {}
        };

        let mut type_of = if self.is_nested() {
            quote! { SchemaType::infer_partial::<#value>() }
        } else {
            quote! { SchemaType::infer::<#value>() }
        };

        if let Some(Expr::Lit(lit)) = &self.args.default {
            let lit_value = match &lit.lit {
                Lit::Str(v) => quote! { LiteralValue::String(#v.into()) },
                Lit::Int(v) => {
                    if v.suffix().starts_with('u') {
                        quote! { LiteralValue::Uint(#v) }
                    } else {
                        quote! { LiteralValue::Int(#v) }
                    }
                }
                Lit::Float(v) => {
                    if v.suffix() == "f32" {
                        quote! { LiteralValue::F32(#v) }
                    } else {
                        quote! { LiteralValue::F64(#v) }
                    }
                }
                Lit::Bool(v) => quote! { LiteralValue::Bool(#v) },
                _ => unimplemented!(),
            };

            type_of = quote! { SchemaType::infer_with_default::<#value>(#lit_value) };
        }

        quote! {
            SchemaField {
                name: Some(#name.into()),
                type_of: #type_of,
                #description
                #deprecated
                #env_var
                #hidden
                #nullable
                ..Default::default()
            }
        }
    }
}

impl<'l> ToTokens for Field<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let value = &self.value_type;

        // Gather all attributes
        let mut attrs = vec![];

        if let Some(serde_meta) = self.get_serde_meta() {
            attrs.push(quote! { #[serde(#serde_meta)] });
        }

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }

        tokens.extend(quote! {
            #(#attrs)*
            pub #name: #value,
        });
    }
}
