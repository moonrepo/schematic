use super::config_type::ConfigType;
use super::variant::TaggedFormat;
use crate::common_schema::*;
use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, ExprPath};

// #[config()]
#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config), supports(struct_named, enum_any))]
pub struct ConfigArgs {
    allow_unknown_fields: bool,
    context: Option<ExprPath>,
    env_prefix: Option<String>,
    file: Option<String>,

    // serde
    rename: Option<String>,
    rename_all: Option<String>,
    serde: SerdeMeta,
}

pub struct Config<'l> {
    pub args: ConfigArgs,
    pub serde_args: SerdeArgs,
    pub attrs: Vec<&'l Attribute>,
    pub name: &'l Ident,
    pub type_of: ConfigType<'l>,
}

impl<'l> Config<'l> {
    pub fn is_enum(&self) -> bool {
        matches!(self.type_of, ConfigType::Enum { .. })
    }

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
            .unwrap_or(if self.is_enum() {
                "kebab-case"
            } else {
                "camelCase"
            })
    }

    pub fn get_tagged_format(&self) -> TaggedFormat {
        if self.args.serde.untagged || self.serde_args.untagged {
            return TaggedFormat::Untagged;
        }

        match (
            self.args
                .serde
                .tag
                .as_ref()
                .or(self.serde_args.tag.as_ref()),
            self.args
                .serde
                .content
                .as_ref()
                .or(self.serde_args.content.as_ref()),
        ) {
            (Some(tag), Some(content)) => {
                TaggedFormat::Adjacent(tag.to_owned(), content.to_owned())
            }
            (Some(tag), None) => TaggedFormat::Internal(tag.to_owned()),
            _ => TaggedFormat::External,
        }
    }

    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![];

        match &self.type_of {
            ConfigType::NamedStruct { .. } => {
                meta.push(quote! { default });

                if !self.args.allow_unknown_fields {
                    meta.push(quote! { deny_unknown_fields });
                }
            }
            ConfigType::Enum { .. } => {
                if let Some(content) = &self.args.serde.content {
                    meta.push(quote! { content = #content });
                } else if let Some(content) = &self.serde_args.content {
                    meta.push(quote! { content = #content });
                }

                if let Some(tag) = &self.args.serde.tag {
                    meta.push(quote! { tag = #tag });
                } else if let Some(tag) = &self.serde_args.tag {
                    meta.push(quote! { tag = #tag });
                }

                if self.args.serde.untagged || self.serde_args.untagged {
                    meta.push(quote! { untagged });
                }
            }
        };

        if let Some(expecting) = &self.args.serde.expecting {
            meta.push(quote! { expecting = #expecting });
        } else if let Some(expecting) = &self.serde_args.expecting {
            meta.push(quote! { expecting = #expecting });
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
        let env_prefix = self.args.env_prefix.as_ref();

        // Generate the partial implementation
        let partial_name = format_ident!("Partial{}", self.name);
        let partial_attrs = self.get_partial_attrs();
        let partial = self.type_of.generate_partial(&partial_name, &partial_attrs);

        tokens.extend(quote! {
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
        let from_partial = self.type_of.generate_from_partial(&partial_name);

        let context = match self.args.context.as_ref() {
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

            let casing_format = self.get_casing_format();
            let tagged_format = self.get_tagged_format();

            let schema = self.type_of.generate_schema(
                name,
                extract_comment(&self.attrs),
                casing_format,
                tagged_format,
            );
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
