use crate::common::FieldValue;
use crate::common::PartialAttr;
use crate::common::macros::ContainerSerdeArgs;
use crate::utils::{extract_common_attrs, format_case, preserve_str_literal};
use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::{Attribute, Expr, ExprPath, Field as NativeField, Type};

// #[serde()]
#[derive(FromAttributes, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct FieldSerdeArgs {
    pub alias: Option<String>,
    pub default: bool,
    pub flatten: bool,
    pub rename: Option<String>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,

    // variant
    pub untagged: bool,
}

impl FieldSerdeArgs {
    pub fn inherit_from_container(&mut self, container: &ContainerSerdeArgs) {
        if !self.default && container.default {
            self.default = true;
        }
    }
}

// #[schema()], #[setting()]
#[derive(FromAttributes, Default)]
#[darling(default, attributes(schema, setting))]
pub struct FieldArgs {
    // schema
    pub exclude: bool,

    // config
    #[darling(with = preserve_str_literal, map = "Some")]
    pub default: Option<Expr>,
    #[cfg(feature = "env")]
    pub env: Option<String>,
    #[cfg(feature = "extends")]
    pub extend: bool,
    pub merge: Option<ExprPath>,
    pub nested: bool,
    #[cfg(feature = "env")]
    pub parse_env: Option<ExprPath>,
    pub required: bool,
    pub transform: Option<ExprPath>,
    #[cfg(feature = "validate")]
    pub validate: Option<Expr>,
    pub partial: PartialAttr,

    // serde
    pub alias: Option<String>,
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
    pub casing_format: String,
    pub name: Option<&'l Ident>, // Named
    pub index: usize,            // Unnamed
    pub value: &'l Type,
    pub value_type: FieldValue<'l>,
    pub env_prefix: Option<String>,
}

impl Field<'_> {
    pub fn from(field: &NativeField) -> Field {
        let args = FieldArgs::from_attributes(&field.attrs).unwrap_or_default();
        let serde_args = FieldSerdeArgs::from_attributes(&field.attrs).unwrap_or_default();

        let field = Field {
            name: field.ident.as_ref(),
            index: 0,
            attrs: extract_common_attrs(&field.attrs),
            casing_format: String::new(),
            value: &field.ty,
            value_type: if args.nested {
                FieldValue::nested(&field.ty)
            } else {
                FieldValue::value(&field.ty)
            },
            args,
            serde_args,
            env_prefix: None,
        };

        if field.args.default.is_some() && field.is_nested() {
            panic!("Cannot use defaults with `nested` configs.");
        }

        if field.is_required() && !field.is_nullable() {
            panic!("Cannot use required with non-optional settings.");
        }

        field
    }

    #[cfg(feature = "schema")]
    pub fn is_excluded(&self) -> bool {
        self.args.exclude
    }

    #[cfg(feature = "extends")]
    pub fn is_extendable(&self) -> bool {
        self.args.extend
    }

    pub fn is_nested(&self) -> bool {
        self.args.nested
    }

    pub fn is_nullable(&self) -> bool {
        self.value_type.is_outer_optional()
    }

    #[cfg(feature = "schema")]
    pub fn is_optional(&self) -> bool {
        self.serde_args.default || self.args.default.is_some()
    }

    pub fn is_required(&self) -> bool {
        self.args.required
    }

    #[cfg(feature = "schema")]
    pub fn is_skipped(&self) -> bool {
        self.args.skip || self.serde_args.skip
    }

    #[cfg(feature = "schema")]
    pub fn get_name_raw(&self) -> &Ident {
        self.name.as_ref().expect("Missing name for field")
    }

    pub fn get_name(&self, casing_format: Option<&str>) -> String {
        let Some(name) = &self.name else {
            return String::new();
        };

        match &self.args.rename {
            Some(local) => local.to_owned(),
            _ => {
                if let Some(serde) = &self.serde_args.rename {
                    serde.to_owned()
                } else if let Some(format) = casing_format {
                    format_case(format, &name.to_string(), false)
                } else {
                    name.to_string()
                }
            }
        }
    }

    pub fn get_aliases(&self) -> Vec<String> {
        let mut aliases = HashSet::new();

        if let Some(alias) = &self.args.alias {
            aliases.insert(alias.to_owned());
        }

        if let Some(alias) = &self.serde_args.alias {
            aliases.insert(alias.to_owned());
        }

        aliases.into_iter().collect()
    }

    pub fn get_env_var(&self) -> Option<String> {
        if self.is_nested() {
            return None;
        }

        #[cfg(feature = "env")]
        if let Some(env_name) = &self.args.env {
            return Some(env_name.to_owned());
        }

        self.env_prefix
            .as_ref()
            .map(|env_prefix| format!("{env_prefix}{}", self.get_name(None)).to_uppercase())
    }

    pub fn get_serde_meta(&self) -> Option<TokenStream> {
        let mut meta = vec![];

        match &self.args.alias {
            Some(alias) => {
                meta.push(quote! { alias = #alias });
            }
            _ => {
                if let Some(alias) = &self.serde_args.alias {
                    meta.push(quote! { alias = #alias });
                }
            }
        }

        if self.args.flatten || self.serde_args.flatten {
            meta.push(quote! { flatten });
        }

        match &self.args.rename {
            Some(rename) => {
                meta.push(quote! { rename = #rename });
            }
            _ => {
                if let Some(rename) = &self.serde_args.rename {
                    meta.push(quote! { rename = #rename });
                }
            }
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

    #[cfg(feature = "schema")]
    pub fn generate_schema_type(&self, as_field: bool) -> TokenStream {
        use crate::utils::{
            extract_comment, extract_deprecated, map_bool_field_quote, map_option_field_quote,
            map_vec_field_quote,
        };
        use syn::Lit;

        let aliases = map_vec_field_quote("aliases", self.get_aliases());
        let hidden = map_bool_field_quote("hidden", self.is_skipped());
        let nullable = map_bool_field_quote("nullable", self.is_nullable());
        let optional = map_bool_field_quote("optional", self.is_optional());
        let comment = map_option_field_quote("comment", extract_comment(&self.attrs));
        let description = map_option_field_quote("description", extract_comment(&self.attrs));
        let deprecated = map_option_field_quote("deprecated", extract_deprecated(&self.attrs));
        let env_var = map_option_field_quote("env_var", self.get_env_var());

        let value = self.value;
        let mut inner_schema = if self.is_nested() {
            quote! { schema.infer_as_nested::<#value>() }
        } else {
            quote! { schema.infer::<#value>() }
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

            inner_schema = quote! { schema.infer_with_default::<#value>(#lit_value) };
        }

        // Struct field (named)
        if as_field {
            let name = self.get_name(Some(&self.casing_format));
            let value = if aliases.is_none()
                && comment.is_none()
                && deprecated.is_none()
                && env_var.is_none()
                && hidden.is_none()
                && nullable.is_none()
                && optional.is_none()
            {
                quote! {
                    SchemaField::new(#inner_schema)
                }
            } else {
                quote! {
                    {
                        let mut field = SchemaField::new(#inner_schema);
                        #aliases
                        #comment
                        #deprecated
                        #env_var
                        #hidden
                        #nullable
                        #optional
                        field
                    }
                }
            };

            quote! {
                (#name.into(), #value)
            }
        }
        // Tuple item (unnamed)
        else {
            #[allow(clippy::collapsible_else_if)]
            if description.is_none() {
                inner_schema
            } else {
                quote! {
                    {
                        let mut schema = #inner_schema;
                        #description
                        schema
                    }
                }
            }
        }
    }
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value_type;

        // Gather all attributes
        let mut attrs = vec![];

        if let Some(serde_meta) = self.get_serde_meta() {
            attrs.push(quote! { #[serde(#serde_meta)] });
        }

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }
        let partial = &self.args.partial;
        attrs.push(quote! {#partial});

        if let Some(name) = &self.name {
            tokens.extend(quote! {
                #(#attrs)*
                pub #name: #value,
            });
        } else {
            tokens.extend(quote! {
                pub #value,
            });
        }
    }
}
