use schematic_core::args::SerdeRenameArg;
use schematic_core::container::Container;
use syn::parse_quote;

mod field_serde {
    use super::*;

    mod native {
        use super::*;

        #[test]
        fn basic() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(alias = "b", flatten, skip)]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.serde_args.alias, vec!["b"]);
            assert!(field.serde_args.flatten);
            assert!(field.serde_args.skip);
        }

        #[test]
        fn multiple_alias() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(alias = "b", alias = "c")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.serde_args.alias, vec!["b", "c"]);
        }

        #[test]
        fn rename() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(rename = "b")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(
                field.serde_args.rename.as_ref().unwrap(),
                &SerdeRenameArg {
                    deserialize: Some("b".into()),
                    serialize: Some("b".into()),
                }
            );
        }

        #[test]
        fn rename_both() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(rename(deserialize = "de_name", serialize = "ser_name"))]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(
                field.serde_args.rename.as_ref().unwrap(),
                &SerdeRenameArg {
                    deserialize: Some("de_name".into()),
                    serialize: Some("ser_name".into()),
                }
            );
        }

        #[test]
        fn skip_de() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(skip_deserializing, skip_deserializing_if = "value")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.serde_args.skip_deserializing);
            assert_eq!(
                field.serde_args.skip_deserializing_if.as_ref().unwrap(),
                "value"
            );
        }

        #[test]
        fn skip_ser() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[serde(skip_serializing, skip_serializing_if = "value")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.serde_args.skip_serializing);
            assert_eq!(
                field.serde_args.skip_serializing_if.as_ref().unwrap(),
                "value"
            );
        }
    }

    mod setting {
        use super::*;

        #[test]
        fn basic() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(alias = "b", flatten, skip)]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.alias, vec!["b"]);
            assert!(field.args.flatten);
            assert!(field.args.skip);
        }

        #[test]
        fn multiple_alias() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(alias = "b", alias = "c")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.alias, vec!["b", "c"]);
        }

        #[test]
        fn rename() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(rename = "b")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(
                field.args.rename.as_ref().unwrap(),
                &SerdeRenameArg {
                    deserialize: Some("b".into()),
                    serialize: Some("b".into()),
                }
            );
        }

        #[test]
        fn rename_both() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(rename(deserialize = "de_name", serialize = "ser_name"))]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(
                field.args.rename.as_ref().unwrap(),
                &SerdeRenameArg {
                    deserialize: Some("de_name".into()),
                    serialize: Some("ser_name".into()),
                }
            );
        }

        #[test]
        fn skip_de() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(skip_deserializing, skip_deserializing_if = "value")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.skip_deserializing);
            assert_eq!(field.args.skip_deserializing_if.as_ref().unwrap(), "value");
        }

        #[test]
        fn skip_ser() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(skip_serializing, skip_serializing_if = "value")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.skip_serializing);
            assert_eq!(field.args.skip_serializing_if.as_ref().unwrap(), "value");
        }
    }
}
