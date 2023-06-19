use super::setting::Setting;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Fields, FieldsNamed, Variant};

pub enum ConfigType<'l> {
    NamedStruct {
        fields: &'l FieldsNamed,
        settings: Vec<Setting<'l>>,
    },
    Enum {
        variants: Vec<&'l Variant>,
    },
}

impl<'l> ConfigType<'l> {
    pub fn generate_default_values(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let mut field_names = vec![];
                let mut default_values = vec![];

                for setting in settings {
                    field_names.push(setting.name);
                    default_values.push(setting.get_default_value());
                }

                quote! {
                    Ok(Self {
                        #(#field_names: #default_values),*
                    })
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_env_values(&self, prefix: Option<&String>) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let env_stmts = settings
                    .iter()
                    .map(|s| s.get_env_statement(prefix))
                    .collect::<Vec<_>>();

                quote! {
                    let mut partial = Self::default();
                    #(#env_stmts)*
                    Ok(partial)
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_extends_from(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                // Validate only 1 setting is using it
                let mut names = vec![];

                for setting in settings {
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
                for setting in settings {
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
                                    self.#name
                                        .as_ref()
                                        .map(|inner| schematic::ExtendsFrom::String(inner.to_owned()))
                                };
                            }
                            "Vec<String>" | "Vec < String >" => {
                                return quote! {
                                    self.#name
                                        .as_ref()
                                        .map(|inner| schematic::ExtendsFrom::List(inner.to_owned()))
                                };
                            }
                            "ExtendsFrom"
                            | "schematic::ExtendsFrom"
                            | "schematic :: ExtendsFrom" => {
                                return quote! {
                                    self.#name.clone()
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

                quote! {
                    None
                }
            }
            ConfigType::Enum { .. } => {
                panic!("Enums do not support `extend`!")
            }
        }
    }

    pub fn generate_finalize(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let finalize_stmts = settings
                    .iter()
                    .map(|s| s.get_finalize_statement())
                    .collect::<Vec<_>>();

                quote! {
                    let mut partial = Self::default_values(context)?;
                    partial.merge(context, self)?;
                    partial.merge(context, Self::env_values()?)?;
                    #(#finalize_stmts)*
                    Ok(partial)
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_merge(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let merge_stmts = settings
                    .iter()
                    .map(|s| s.get_merge_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#merge_stmts)*
                    Ok(())
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_validate(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let validate_stmts = settings
                    .iter()
                    .map(|s| s.get_validate_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#validate_stmts)*
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_from_partial(&self) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let mut field_names = vec![];
                let mut from_partial_values = vec![];

                for setting in settings {
                    field_names.push(setting.name);
                    from_partial_values.push(setting.get_from_partial_value());
                }

                quote! {
                    Self {
                        #(#field_names: #from_partial_values),*
                    }
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_schema(
        &self,
        config_name: &Ident,
        description: Option<String>,
        casing_format: &str,
    ) -> TokenStream {
        let config_name = config_name.to_string();
        let description = if let Some(comment) = description {
            quote! {
                structure.description = Some(#comment.into());
            }
        } else {
            quote! {}
        };

        match self {
            ConfigType::NamedStruct { settings, .. } => {
                let schema_types = settings
                    .iter()
                    .map(|s| s.get_schema_type(casing_format))
                    .collect::<Vec<_>>();

                quote! {
                    let mut structure = StructType {
                        name: Some(#config_name.into()),
                        fields: vec![
                            #(#schema_types),*
                        ],
                        ..Default::default()
                    };

                    #description

                    SchemaType::Struct(structure)
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_partial(&self, partial_name: &Ident) -> TokenStream {
        match self {
            ConfigType::NamedStruct { settings, .. } => {
                quote! {
                    pub struct #partial_name {
                        #(#settings)*
                    }
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }

    pub fn generate_partial_schema(
        &self,
        config_name: &Ident,
        _partial_name: &Ident,
    ) -> TokenStream {
        match self {
            ConfigType::NamedStruct { .. } => {
                quote! {
                    let mut schema = #config_name::generate_schema();
                    schematic::internal::partialize_schema(&mut schema);
                    schema
                }
            }
            ConfigType::Enum { variants } => todo!(),
        }
    }
}
