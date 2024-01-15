pub mod container;
pub mod field;
pub mod field_value;
pub mod variant;

use crate::common::Macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub struct ConfigMacro<'l>(pub Macro<'l>);

impl<'l> ToTokens for ConfigMacro<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cfg = &self.0;
        let name = cfg.name;

        // Generate the partial implementation
        let partial_name = format_ident!("Partial{}", cfg.name);
        let partial_attrs = cfg.get_partial_attrs();
        let partial = cfg.type_of.generate_partial(&partial_name, &partial_attrs);

        tokens.extend(quote! {
            #partial
        });

        // Generate implementations
        let meta = cfg.get_meta_struct();
        let default_values = cfg.type_of.generate_default_values();
        let env_values = cfg.type_of.generate_env_values();
        let extends_from = cfg.type_of.generate_extends_from();
        let finalize = cfg.type_of.generate_finalize();
        let merge = cfg.type_of.generate_merge();
        let validate = cfg.type_of.generate_validate();
        let from_partial = cfg.type_of.generate_from_partial(&partial_name);

        let context = match cfg.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                type Context = #context;

                fn default_values(context: &Self::Context) -> Result<Option<Self>, schematic::ConfigError> {
                    #default_values
                }

                fn env_values() -> Result<Option<Self>, schematic::ConfigError> {
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
                    finalize: bool,
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

                    let defaults = <<Self as schematic::Config>::Partial as schematic::PartialConfig>::default_values(&context).unwrap().unwrap_or_default();

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

            let schema = cfg
                .type_of
                .generate_schema(cfg.get_name(), extract_comment(&cfg.attrs));
            let partial_schema = cfg.type_of.generate_partial_schema(name, &partial_name);

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
