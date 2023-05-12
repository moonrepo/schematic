use super::setting::Setting;
use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

// #[config()]
#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(config))]
pub struct ConfigArgs {
    // serde
    rename: Option<String>,
    rename_all: Option<String>,
}

impl ConfigArgs {
    pub fn get_serde_meta(&self) -> TokenStream {
        let mut meta = vec![quote! { default }, quote! { deny_unknown_fields }];

        if let Some(rename) = &self.rename {
            meta.push(quote! { rename = #rename });
        }

        let rename_all = self.rename_all.as_deref().unwrap_or("camelCase");

        meta.push(quote! { rename_all = #rename_all });

        quote! {
            #(#meta),*
        }
    }
}

pub struct Config<'l> {
    pub args: ConfigArgs,
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
                "Only 1 setting may use `extends`, found: {}",
                names.join(", ")
            );
        }

        // Loop again and generate the necessary code
        for setting in &self.settings {
            if setting.is_extendable() {
                let name = setting.name;
                let type_of = format!("{}", setting.value.to_token_stream());

                // Janky but works!
                match type_of.as_str() {
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
                    _ => {
                        panic!(
                            "Only `String`, `Vec<String>`, or `ExtendsFrom` are supported when using `extends` for {name}."
                        );
                    }
                };
            }
        }

        quote! {}
    }

    pub fn get_partial_attrs(&self) -> Vec<TokenStream> {
        let serde_meta = self.args.get_serde_meta();
        let attrs = vec![quote! { #[serde(#serde_meta) ]}];

        #[cfg(feature = "json_schema")]
        {
            attrs.push(quote! { #[derive(schemars::JsonSchema)] });
        }

        #[cfg(feature = "typescript")]
        {
            attrs.push(quote! { #[derive(ts_rs::TS)] });
        }

        attrs
    }
}

impl<'l> ToTokens for Config<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.name;

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
        let mut default_values = vec![];
        let mut from_values = vec![];
        let mut merge_stmts = vec![];
        let extends_from = self.extends_from();

        for setting in &self.settings {
            field_names.push(setting.name);
            default_values.push(setting.get_default_value());
            from_values.push(setting.get_from_value());
            merge_stmts.push(setting.get_merge_statement());
        }

        let token = quote! {
            #[automatically_derived]
            impl schematic::PartialConfig for #partial_name {
                fn default_values() -> Result<Self, schematic::ConfigError> {
                    Ok(Self {
                        #(#field_names: #default_values),*
                    })
                }

                fn extends_from(&self) -> Option<schematic::ExtendsFrom> {
                    #extends_from
                    None
                }

                fn merge(&mut self, mut next: Self) {
                    #(#merge_stmts)*
                }
            }
        };

        tokens.extend(token);

        let token = quote! {
            #[automatically_derived]
            impl schematic::Config for #name {
                type Partial = #partial_name;

                fn from_partial(partial: Self::Partial) -> Self {
                    Self {
                        #(#field_names: #from_values),*
                    }
                }
            }
        };

        tokens.extend(token);
    }
}
