use darling::{FromDeriveInput, FromMeta};
use schematic_core::args::*;
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
