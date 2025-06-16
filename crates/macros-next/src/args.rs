use darling::ast::NestedMeta;
use darling::{FromAttributes, FromDeriveInput, FromMeta};
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Attribute, Data, DeriveInput, ExprPath, Fields};

#[derive(Clone, Copy)]
pub enum SerdeIoDirection {
    From, // read / deserialize
    To,   // write / serialize
}

#[derive(Clone)]
pub enum SerdeTagFormat {
    Untagged,
    External,
    Internal(String),
    Adjacent(String, String),
    // Special case for unit only enums
    Unit,
}

// #[serde(rename = "name")]
// #[serde(rename(deserialize = "de_name", serialize = "ser_name"))]
#[derive(Debug, Default, PartialEq)]
pub struct SerdeRenameArg {
    deserialize: Option<String>,
    serialize: Option<String>,
}

impl FromMeta for SerdeRenameArg {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self {
            deserialize: Some(value.into()),
            serialize: Some(value.into()),
        })
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        #[derive(Default, FromMeta)]
        #[darling(default)]
        struct Rename {
            deserialize: Option<String>,
            serialize: Option<String>,
        }

        impl From<Rename> for SerdeRenameArg {
            fn from(value: Rename) -> Self {
                Self {
                    deserialize: value.deserialize,
                    serialize: value.serialize,
                }
            }
        }

        Rename::from_list(items).map(SerdeRenameArg::from)
    }
}

impl SerdeRenameArg {
    // pub fn get_name(&self, dir: SerdeIoDirection) -> Option<&str> {
    //     match dir {
    //         SerdeIoDirection::From => self.deserialize.as_deref(),
    //         SerdeIoDirection::To => self.serialize.as_deref(),
    //     }
    // }

    pub fn get_meta(&self, key: &str) -> TokenStream {
        match (self.deserialize.as_deref(), self.serialize.as_deref()) {
            (Some(de), Some(ser)) => {
                if de == ser {
                    quote! { #key = #de }
                } else {
                    quote! { #key(deserialize = #de, serialize = #ser) }
                }
            }
            (None, Some(ser)) => quote! { #key(serialize = #ser) },
            (Some(de), None) => quote! { #key(deserialize = #de) },
            _ => quote! {},
        }
    }
}

// #[serde()]
#[derive(Default, FromDeriveInput)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeContainerArgs {
    pub default: bool,
    pub deny_unknown_fields: bool,

    // struct
    pub rename: Option<SerdeRenameArg>,
    pub rename_all: Option<SerdeRenameArg>,
    pub rename_all_fields: Option<SerdeRenameArg>,

    // enum
    pub content: Option<String>,
    pub expecting: Option<String>,
    pub tag: Option<String>,
    pub untagged: bool,
}

// #[serde()]
#[derive(Default, FromAttributes)]
#[darling(default, allow_unknown_fields, attributes(serde))]
pub struct SerdeFieldArgs {
    #[darling(multiple)]
    pub alias: Vec<String>,
    pub default: bool,
    pub flatten: bool,
    pub rename: Option<SerdeRenameArg>,
    pub skip: bool,
    pub skip_deserializing: bool,
    pub skip_deserializing_if: Option<String>,
    pub skip_serializing: bool,
    pub skip_serializing_if: Option<String>,

    // variant
    pub other: bool,
    pub untagged: bool,
}

// // #[config()], #[schematic()]
// #[derive(FromDeriveInput, Default)]
// #[darling(
//     default,
//     attributes(config, schematic),
//     supports(struct_named, enum_any)
// )]
// pub struct MacroArgs {
//     // config
//     pub allow_unknown_fields: bool,
//     pub context: Option<ExprPath>,
//     pub partial: PartialAttr,
//     #[cfg(feature = "env")]
//     pub env_prefix: Option<String>,

//     // serde
//     pub rename: Option<String>,
//     pub rename_all: Option<String>,
//     pub rename_all_fields: Option<String>,
//     pub serde: SerdeMeta,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use darling::FromMeta;
    use syn::parse_quote;

    mod serde_rename_arg {
        use super::*;

        #[test]
        fn both_value_string() {
            let meta = SerdeRenameArg::from_string("name").unwrap();

            assert_eq!(
                meta,
                SerdeRenameArg {
                    deserialize: Some("name".into()),
                    serialize: Some("name".into()),
                }
            );
        }

        #[test]
        fn both_value() {
            let meta = SerdeRenameArg::from_list(&[
                parse_quote! {
                    deserialize = "de_name"
                },
                parse_quote! {
                    serialize = "ser_name"
                },
            ])
            .unwrap();

            assert_eq!(
                meta,
                SerdeRenameArg {
                    deserialize: Some("de_name".into()),
                    serialize: Some("ser_name".into()),
                }
            );
        }

        #[test]
        fn de_value() {
            let meta = SerdeRenameArg::from_list(&[parse_quote! {
                deserialize = "de_name"
            }])
            .unwrap();

            assert_eq!(
                meta,
                SerdeRenameArg {
                    deserialize: Some("de_name".into()),
                    serialize: None,
                }
            );
        }

        #[test]
        fn ser_value() {
            let meta = SerdeRenameArg::from_list(&[parse_quote! {
                serialize = "ser_name"
            }])
            .unwrap();

            assert_eq!(
                meta,
                SerdeRenameArg {
                    deserialize: None,
                    serialize: Some("ser_name".into()),
                }
            );
        }
    }

    mod serde_container {
        use super::*;

        #[test]
        fn normal_args() {
            let container = SerdeContainerArgs::from_derive_input(&parse_quote! {
                #[serde(default, deny_unknown_fields)]
                struct Example;
            })
            .unwrap();

            assert!(container.default);
            assert!(container.deny_unknown_fields);
        }

        #[test]
        fn enum_tagged_args() {
            let container = SerdeContainerArgs::from_derive_input(&parse_quote! {
                #[serde(tag = "tag", content = "content")]
                struct Example;
            })
            .unwrap();

            assert_eq!(container.content.unwrap(), "content");
            assert_eq!(container.tag.unwrap(), "tag");
            assert!(container.expecting.is_none());
            assert!(!container.untagged);
        }

        #[test]
        fn enum_untagged_args() {
            let container = SerdeContainerArgs::from_derive_input(&parse_quote! {
                #[serde(untagged, expecting = "expecting")]
                struct Example;
            })
            .unwrap();

            assert!(container.content.is_none());
            assert!(container.tag.is_none());
            assert_eq!(container.expecting.unwrap(), "expecting");
            assert!(container.untagged);
        }

        #[test]
        fn rename_args() {
            let container = SerdeContainerArgs::from_derive_input(&parse_quote! {
                #[serde(
                    rename = "name",
                    rename_all(deserialize = "de_name"),
                    rename_all_fields(serialize = "ser_name")
                )]
                struct Example;
            })
            .unwrap();

            assert_eq!(
                container.rename.unwrap(),
                SerdeRenameArg {
                    deserialize: Some("name".into()),
                    serialize: Some("name".into()),
                }
            );
            assert_eq!(
                container.rename_all.unwrap(),
                SerdeRenameArg {
                    deserialize: Some("de_name".into()),
                    serialize: None,
                }
            );

            assert_eq!(
                container.rename_all_fields.unwrap(),
                SerdeRenameArg {
                    deserialize: None,
                    serialize: Some("ser_name".into()),
                }
            );
        }
    }
}
