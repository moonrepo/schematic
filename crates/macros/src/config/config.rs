use super::setting::Setting;
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
    pub settings: Vec<Setting<'l>>,
}

impl<'l> Config<'l> {
    pub fn extends_from(&self) -> TokenStream {
        // Validate only 1 setting is using it
        let mut names = vec![];

        for setting in &self.settings {
            if setting.is_extendable() {
                names.push(setting.name.to_string());
            }
        }

        if names.len() > 1 {
            panic!(
                "Only 1 setting may use `extend`, found: {}",
                names.join(", ")
            );
        }

        // Loop again and generate the necessary code
        for setting in &self.settings {
            if !setting.is_extendable() {
                continue;
            }

            if let Some(inner_type) = setting.value_type.get_inner_type() {
                let name = setting.name;
                let value = format!("{}", inner_type.to_token_stream());

                // Janky but works!
                match value.as_str() {
                    "String" => {
                        return quote! {
                            if let Some(value) = self.#name.as_ref() {
                                return Some(schematic::ExtendsFrom::String(value.clone()));
                            }
                        };
                    }
                    "Vec<String>" | "Vec < String >" => {
                        return quote! {
                            if let Some(value) = self.#name.as_ref() {
                                return Some(schematic::ExtendsFrom::List(value.clone()));
                            }
                        };
                    }
                    "ExtendsFrom" | "schematic::ExtendsFrom" | "schematic :: ExtendsFrom" => {
                        return quote! {
                            if let Some(value) = self.#name.as_ref() {
                                return Some(value.clone());
                            }
                        };
                    }
                    inner => {
                        let inner = inner.to_string();

                        panic!(
                            "Only `String`, `Vec<String>`, or `ExtendsFrom` are supported when using `extend` for {name}. Received `{inner}`."
                        );
                    }
                };
            }
        }

        quote! {}
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

        #[cfg(feature = "json_schema")]
        {
            attrs.push(quote! { #[derive(schemars::JsonSchema)] });
        }

        attrs
    }
}

impl<'l> ToTokens for Config<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;
        let casing_format = self.get_casing_format();

        let context = match self.args.context.as_ref() {
            Some(ctx) => quote! { #ctx },
            None => quote! { () },
        };

        // Generate the partial struct
        let partial_name = format_ident!("Partial{}", self.name);
        let partial_attrs = self.get_partial_attrs();
        let partial_fields = &self.settings;

        let token = quote! {
            #[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
            #(#partial_attrs)*
            pub struct #partial_name {
                #(#partial_fields)*
            }
        };

        tokens.extend(token);

        // Generate implementations
        let mut field_names = vec![];
        let env_prefix = self.args.env_prefix.as_ref();
        let extends_from = self.extends_from();

        let mut default_values = vec![];
        let mut from_partial_values = vec![];
        let mut schema_types = vec![];

        let mut env_stmts = vec![];
        let mut finalize_stmts = vec![];
        let mut merge_stmts = vec![];
        let mut validate_stmts = vec![];

        for setting in &self.settings {
            field_names.push(setting.name);

            default_values.push(setting.get_default_value());
            from_partial_values.push(setting.get_from_partial_value());
            schema_types.push(setting.get_schema_type(casing_format));

            env_stmts.push(setting.get_env_statement(env_prefix));
            finalize_stmts.push(setting.get_finalize_statement());
            merge_stmts.push(setting.get_merge_statement());
            validate_stmts.push(setting.get_validate_statement());
        }

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                type Context = #context;

                fn default_values(context: &Self::Context) -> Result<Self, schematic::ConfigError> {
                    Ok(Self {
                        #(#field_names: #default_values),*
                    })
                }

                fn env_values() -> Result<Self, schematic::ConfigError> {
                    let mut partial = Self::default();
                    #(#env_stmts)*
                    Ok(partial)
                }

                fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                    #extends_from
                    None
                }

                fn finalize(self, context: &Self::Context) -> Result<Self, schematic::ConfigError> {
                    let mut partial = Self::default_values(context)?;
                    partial.merge(context, self)?;
                    partial.merge(context, Self::env_values()?)?;
                    #(#finalize_stmts)*
                    Ok(partial)
                }

                fn merge(
                    &mut self,
                    context: &Self::Context,
                    mut next: Self,
                ) -> Result<(), schematic::ConfigError> {
                    #(#merge_stmts)*
                    Ok(())
                }

                fn validate_with_path(
                    &self,
                    context: &Self::Context,
                    path: schematic::SettingPath
                ) -> Result<(), schematic::ValidatorError> {
                    let mut errors: Vec<schematic::ValidateErrorType> = vec![];

                    #(#validate_stmts)*

                    if !errors.is_empty() {
                        return Err(schematic::ValidatorError {
                            errors,
                            path,
                        });
                    }

                    Ok(())
                }
            }
        });

        let meta = self.get_meta_struct();

        tokens.extend(quote! {
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
                    Self {
                        #(#field_names: #from_partial_values),*
                    }
                }
            }
        });

        #[cfg(feature = "schema")]
        {
            let config_name = name.to_string();

            tokens.extend(quote! {
                #[automatically_derived]
                impl schematic::Schematic for #name {
                    fn generate_schema() -> schematic::SchemaType {
                        use schematic::schema::*;

                        SchemaType::Struct(StructType {
                            name: Some(#config_name.into()),
                            fields: vec![
                                #(#schema_types),*
                            ],
                            ..Default::default()
                        })
                    }
                }
            });
        }

        #[cfg(not(feature = "schema"))]
        {
            tokens.extend(quote! {
                #[automatically_derived]
                impl schematic::Schematic for #name {}
            });
        }
    }
}
