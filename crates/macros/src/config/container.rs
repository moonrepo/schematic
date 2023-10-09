use crate::common::Container;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

impl<'l> Container<'l> {
    pub fn generate_default_values(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let mut setting_names = vec![];
                let mut default_values = vec![];

                for setting in settings {
                    setting_names.push(setting.name);
                    default_values.push(setting.generate_default_value());
                }

                quote! {
                    Ok(Some(Self {
                        #(#setting_names: #default_values),*
                    }))
                }
            }
            Self::Enum { variants } => {
                let default_variant = variants.iter().find(|v| v.is_default());

                if let Some(variant) = default_variant {
                    let default_value = variant.generate_default_value();

                    quote! {
                        Ok(Some(Self::#default_value))
                    }
                } else {
                    quote! {
                        Ok(None)
                    }
                }
            }
        }
    }

    pub fn generate_env_values(&self, prefix: Option<&String>) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let env_stmts = settings
                    .iter()
                    .filter_map(|s| s.generate_env_statement(prefix))
                    .collect::<Vec<_>>();

                if env_stmts.is_empty() {
                    quote! {
                        Ok(None)
                    }
                } else {
                    quote! {
                        let mut partial = Self::default();
                        #(#env_stmts)*
                        Ok(Some(partial))
                    }
                }
            }
            Self::Enum { .. } => {
                quote! {
                    Ok(None)
                }
            }
        }
    }

    pub fn generate_extends_from(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
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

                quote! { None }
            }
            Self::Enum { .. } => {
                quote! { None }
            }
        }
    }

    pub fn generate_finalize(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let finalize_stmts = settings
                    .iter()
                    .map(|s| s.generate_finalize_statement())
                    .collect::<Vec<_>>();

                quote! {
                    let mut partial = Self::default();

                    if let Some(data) = Self::default_values(context)? {
                        partial.merge(context, data)?;
                    }

                    partial.merge(context, self)?;

                    if let Some(data) = Self::env_values()? {
                        partial.merge(context, data)?;
                    }

                    #(#finalize_stmts)*

                    Ok(partial)
                }
            }
            Self::Enum { variants } => {
                if self.has_nested() {
                    let finalize_stmts = variants
                        .iter()
                        .flat_map(|s| s.generate_finalize_statement())
                        .collect::<Vec<_>>();

                    quote! {
                        Ok(match self {
                            #(#finalize_stmts)*
                            _ => self
                        })
                    }
                } else {
                    quote! {
                        Ok(self)
                    }
                }
            }
        }
    }

    pub fn generate_merge(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let merge_stmts = settings
                    .iter()
                    .map(|s| s.generate_merge_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#merge_stmts)*
                    Ok(())
                }
            }
            Self::Enum { variants } => {
                let merge_stmts = variants
                    .iter()
                    .filter_map(|s| s.generate_merge_statement())
                    .collect::<Vec<_>>();

                if merge_stmts.is_empty() {
                    quote! {
                        *self = next;
                        Ok(())
                    }
                } else {
                    quote! {
                        match self {
                            #(#merge_stmts)*
                            _ => {
                                *self = next;
                            }
                        };
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn generate_validate(&self) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let validate_stmts = settings
                    .iter()
                    .map(|s| s.generate_validate_statement())
                    .collect::<Vec<_>>();

                quote! {
                    #(#validate_stmts)*
                }
            }
            Self::Enum { variants } => {
                let validate_stmts = variants
                    .iter()
                    .filter_map(|s| s.generate_validate_statement())
                    .collect::<Vec<_>>();

                if validate_stmts.is_empty() {
                    quote! {}
                } else {
                    quote! {
                        match self {
                            #(#validate_stmts)*
                            _ => {}
                        };
                    }
                }
            }
        }
    }

    pub fn generate_from_partial(&self, partial_name: &Ident) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                let mut setting_names = vec![];
                let mut from_partial_values = vec![];

                for setting in settings {
                    setting_names.push(setting.name);
                    from_partial_values.push(setting.generate_from_partial_value());
                }

                quote! {
                    Self {
                        #(#setting_names: #from_partial_values),*
                    }
                }
            }
            Self::Enum { variants } => {
                let from_partial_values = variants
                    .iter()
                    .map(|s| s.generate_from_partial_value(partial_name))
                    .collect::<Vec<_>>();

                quote! {
                    match partial {
                        #(#from_partial_values)*
                    }
                }
            }
        }
    }

    pub fn generate_partial(
        &self,
        partial_name: &Ident,
        partial_attrs: &[TokenStream],
    ) -> TokenStream {
        match self {
            Self::NamedStruct {
                fields: settings, ..
            } => {
                quote! {
                    #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
                    #(#partial_attrs)*
                    pub struct #partial_name {
                        #(#settings)*
                    }
                }
            }
            Self::Enum { variants } => {
                let default_variant = variants
                    .iter()
                    .find(|v| v.is_default())
                    .or_else(|| variants.first());

                let default_impl = if let Some(default) = default_variant {
                    let value = default.generate_default_value();

                    quote! { Self::#value }
                } else {
                    quote! { panic!("No variant has been marked as default!"); }
                };

                quote! {
                    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
                    #(#partial_attrs)*
                    pub enum #partial_name {
                        #(#variants)*
                    }

                    impl Default for #partial_name {
                        fn default() -> Self {
                            #default_impl
                        }
                    }
                }
            }
        }
    }

    pub fn generate_partial_schema(
        &self,
        config_name: &Ident,
        _partial_name: &Ident,
    ) -> TokenStream {
        match self {
            Self::NamedStruct { .. } => {
                quote! {
                    let mut schema = #config_name::generate_schema();
                    schematic::internal::partialize_schema(&mut schema, true);
                    schema
                }
            }
            Self::Enum { .. } => {
                quote! {
                    let mut schema = #config_name::generate_schema();
                    schematic::internal::partialize_schema(&mut schema, true);
                    schema
                }
            }
        }
    }
}
