use super::config_type::ConfigType;
use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, ExprPath};

// #[serde()]
#[derive(FromDeriveInput, Default)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeArgs {
    rename: Option<String>,
    rename_all: Option<String>,
}

// #[config()]
#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config), supports(struct_named))]
pub struct ConfigArgs {
    allow_unknown_fields: bool,
    context: Option<ExprPath>,
    env_prefix: Option<String>,
    file: Option<String>,

    // serde
    rename: Option<String>,
    rename_all: Option<String>,
}

pub struct Config<'l> {
    pub args: ConfigArgs,
    pub serde_args: SerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub type_of: ConfigType<'l>,
}

impl<'l> Config<'l> {
    pub fn get_meta_struct(&self) -> TokenStream {
        let name = if let Some(rename) = &self.args.rename {
            rename.to_string()
        } else {
            format!("{}", self.name)
        };

        quote! {
            schematic::Meta {
                name: #name,
            }
        }
    }

    pub fn get_casing_format(&self) -> &str {
        self.args
            .rename_all
            .as_deref()
            .or(self.serde_args.rename_all.as_deref())
            .unwrap_or("camelCase")
    }

    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![quote! { default }];

        if !self.args.allow_unknown_fields {
            meta.push(quote! { deny_unknown_fields });
        }

        if let Some(rename) = &self.args.rename {
            meta.push(quote! { rename = #rename });
        } else if let Some(rename) = &self.serde_args.rename {
            meta.push(quote! { rename = #rename });
        }

        let rename_all = self.get_casing_format();

        meta.push(quote! { rename_all = #rename_all });

        quote! {
            #(#meta),*
        }
    }

    pub fn get_partial_attrs(&self) -> Vec<TokenStream> {
        let serde_meta = self.get_serde_meta();
        let mut attrs = vec![quote! { #[serde(#serde_meta) ]}];

        for attr in &self.attrs {
            attrs.push(quote! { #attr });
        }

        attrs
    }
}

impl<'l> ToTokens for Config<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let casing_format = self.get_casing_format();
        let env_prefix = self.args.env_prefix.as_ref();

        let context = match self.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        // Generate the partial implementation
        let partial_name = format_ident!("Partial{}", self.name);
        let partial_attrs = self.get_partial_attrs();
        let partial = self.type_of.generate_partial(&partial_name);

        tokens.extend(quote! {
            #[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
            #(#partial_attrs)*
            #partial
        });

        // Generate implementations
        let meta = self.get_meta_struct();
        let default_values = self.type_of.generate_default_values();
        let env_values = self.type_of.generate_env_values(env_prefix);
        let extends_from = self.type_of.generate_extends_from();
        let finalize = self.type_of.generate_finalize();
        let merge = self.type_of.generate_merge();
        let validate = self.type_of.generate_validate();
        let from_partial = self.type_of.generate_from_partial();

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                type Context = #context;

                fn default_values(context: &Self::Context) -> Result<Self, schematic::ConfigError> {
                    #default_values
                }

                fn env_values() -> Result<Self, schematic::ConfigError> {
                    #env_values
                }

                fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                    #extends_from
                }

                fn finalize(self, context: &Self::Context) -> Result<Self, schematic::ConfigError> {
                    #finalize
                }

                fn merge(
                    &mut self,
                    context: &Self::Context,
                    mut next: Self,
                ) -> Result<(), schematic::ConfigError> {
                    #merge
                }

                fn validate_with_path(
                    &self,
                    context: &Self::Context,
                    path: schematic::Path
                ) -> Result<(), schematic::ValidatorError> {
                    let mut errors: Vec<schematic::ValidateErrorType> = vec![];

                    #validate

                    if !errors.is_empty() {
                        return Err(schematic::ValidatorError {
                            errors,
                            path,
                        });
                    }

                    Ok(())
                }
            }

            #[automatically_derived]
            impl Default for #name {
                fn default() -> Self {
                    let context = <<Self as schematic::Config>::Partial as schematic::PartialConfig>::Context::default();

                    let defaults = <<Self as schematic::Config>::Partial as schematic::PartialConfig>::default_values(&context).unwrap();

                    <Self as schematic::Config>::from_partial(defaults)
                }
            }

            #[automatically_derived]
            impl schematic::Config for #name {
                type Partial = #partial_name;

                const META: schematic::Meta = #meta;

                fn from_partial(partial: Self::Partial) -> Self {
                    #from_partial
                }
            }
        });

        #[cfg(feature = "schema")]
        {
            use crate::utils::extract_comment;

            let schema =
                self.type_of
                    .generate_schema(name, extract_comment(&self.attrs), casing_format);
            let partial_schema = self.type_of.generate_partial_schema(name, &partial_name);

            tokens.extend(quote! {
                #[automatically_derived]
                impl schematic::Schematic for #name {
                    fn generate_schema() -> schematic::SchemaType {
                        use schematic::schema::*;
                        #schema
                    }
                }

                #[automatically_derived]
                impl schematic::Schematic for #partial_name {
                    fn generate_schema() -> schematic::SchemaType {
                        use schematic::schema::*;
                        #partial_schema
                    }
                }
            });
        }

        #[cfg(not(feature = "schema"))]
        {
            tokens.extend(quote! {
                #[automatically_derived]
                impl schematic::Schematic for #name {}

                #[automatically_derived]
                impl schematic::Schematic for #partial_name {}
            });
        }
    }
}
