pub mod container;
pub mod field;
pub mod field_value;
pub mod variant;

use crate::common::Macro;
use crate::utils::instrument_quote;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

pub struct ConfigMacro<'l>(pub Macro<'l>);

impl ToTokens for ConfigMacro<'_> {
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
        let default_values = cfg.type_of.generate_default_values();
        let finalize = cfg.type_of.generate_finalize();
        let merge = cfg.type_of.generate_merge();
        let from_partial = cfg.type_of.generate_from_partial(&partial_name);
        let settings_metadata = cfg.type_of.generate_settings_metadata();
        let instrument = instrument_quote();

        let context = match cfg.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        let env_method = if cfg!(feature = "env") {
            let env_values = cfg.type_of.generate_env_values();

            quote! {
                #instrument
                fn env_values() -> Result<Option<Self>, schematic::ConfigError> {
                    use schematic::internal::*;
                    #env_values
                }
            }
        } else {
            quote! {}
        };

        let extends_method = if cfg!(feature = "extends") {
            let extends_from = cfg.type_of.generate_extends_from();

            quote! {
                #instrument
                fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                    #extends_from
                }
            }
        } else {
            quote! {}
        };

        let validate_method = if cfg!(feature = "validate") {
            let validate = cfg.type_of.generate_validate();

            quote! {
                #instrument
                fn validate_with_path(
                    &self,
                    context: &Self::Context,
                    finalize: bool,
                    path: schematic::Path
                ) -> Result<(), Vec<schematic::ValidateError>> {
                    let mut errors = vec![];

                    #validate

                    if !errors.is_empty() {
                        return Err(errors);
                    }

                    Ok(())
                }
            }
        } else {
            quote! {}
        };

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                type Context = #context;

                #instrument
                fn default_values(context: &Self::Context) -> Result<Option<Self>, schematic::ConfigError> {
                    use schematic::internal::*;
                    #default_values
                }

                #env_method

                #extends_method

                #instrument
                fn finalize(self, context: &Self::Context) -> Result<Self, schematic::ConfigError> {
                    #finalize
                }

                #instrument
                fn merge(
                    &mut self,
                    context: &Self::Context,
                    mut next: Self,
                ) -> Result<(), schematic::ConfigError> {
                    use schematic::internal::*;
                    #merge
                }

                #validate_method
            }

            #[automatically_derived]
            impl Default for #name {
                #instrument
                fn default() -> Self {
                    let context = <<Self as schematic::Config>::Partial as schematic::PartialConfig>::Context::default();

                    let defaults = <<Self as schematic::Config>::Partial as schematic::PartialConfig>::default_values(&context).unwrap().unwrap_or_default();

                    <Self as schematic::Config>::from_partial(defaults)
                }
            }

            #[automatically_derived]
            impl schematic::Config for #name {
                type Partial = #partial_name;

                #instrument
                fn from_partial(partial: Self::Partial) -> Self {
                    #from_partial
                }

                #instrument
                fn settings() -> schematic::ConfigSettingMap {
                    #settings_metadata
                }
            }
        });

        #[cfg(feature = "schema")]
        {
            let schema_name = cfg.get_name();
            let schema_impl = cfg.type_of.generate_schema(&cfg.attrs);

            let partial_schema_name = partial_name.to_string();
            let partial_schema_impl = cfg.type_of.generate_partial_schema(name, &partial_name);

            tokens.extend(quote! {
                #[automatically_derived]
                impl schematic::Schematic for #name {
                    fn schema_name() -> Option<String> {
                        Some(#schema_name.into())
                    }

                    #instrument
                    fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
                        use schematic::schema::*;

                        #schema_impl
                    }
                }

                #[automatically_derived]
                impl schematic::Schematic for #partial_name {
                    fn schema_name() -> Option<String> {
                        Some(#partial_schema_name.into())
                    }

                    #instrument
                    fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
                        #partial_schema_impl
                        schema
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
