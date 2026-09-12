mod utils;

use schematic_core::args::*;
use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod container {
    use super::*;

    #[test]
    fn basic_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(allow_unknown_fields, env_prefix = "PREFIX_", context = ExampleContext)]
            struct Example {}
        });

        assert!(container.args.allow_unknown_fields);
        assert!(container.args.context.is_some());
        assert_eq!(container.args.env_prefix.as_ref().unwrap(), "PREFIX_");
    }

    #[test]
    fn rename_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(
                rename = "name",
                rename_all(deserialize = "de_name"),
                rename_all_fields(serialize = "ser_name")
            )]
            struct Example {}
        });

        assert_eq!(
            container.args.rename.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: Some("name".into()),
                serialize: Some("name".into()),
            }
        );
        assert_eq!(
            container.args.rename_all.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: Some("de_name".into()),
                serialize: None,
            }
        );

        assert_eq!(
            container.args.rename_all_fields.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: None,
                serialize: Some("ser_name".into()),
            }
        );
    }

    #[test]
    fn serde_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(default, deny_unknown_fields)]
            struct Example {}
        });

        assert!(container.serde_args.default.is_enabled());
        assert!(container.serde_args.deny_unknown_fields);
    }

    #[test]
    fn serde_tagged_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(tag = "tag", content = "content")]
            struct Example {}
        });

        assert_eq!(container.serde_args.content.as_ref().unwrap(), "content");
        assert_eq!(container.serde_args.tag.as_ref().unwrap(), "tag");
        assert!(container.serde_args.expecting.is_none());
        assert!(!container.serde_args.untagged);
    }

    #[test]
    fn serde_untagged_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(untagged, expecting = "expecting")]
            struct Example {}
        });

        assert!(container.serde_args.content.is_none());
        assert!(container.serde_args.tag.is_none());
        assert_eq!(
            container.serde_args.expecting.as_ref().unwrap(),
            "expecting"
        );
        assert!(container.serde_args.untagged);
    }

    #[test]
    fn serde_rename_args() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(
                rename = "name",
                rename_all(deserialize = "de_name"),
                rename_all_fields(serialize = "ser_name")
            )]
            struct Example {}
        });

        assert_eq!(
            container.serde_args.rename.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: Some("name".into()),
                serialize: Some("name".into()),
            }
        );
        assert_eq!(
            container.serde_args.rename_all.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: Some("de_name".into()),
                serialize: None,
            }
        );

        assert_eq!(
            container.serde_args.rename_all_fields.as_ref().unwrap(),
            &SerdeRenameArg {
                deserialize: None,
                serialize: Some("ser_name".into()),
            }
        );
    }
}

mod settings {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn supports() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: String,
                    b: i32,
                    #[setting(env = "C_VAR")]
                    c: bool,
                    d: Option<String>,
                    e: Vec<u8>,
                    f: std::collections::HashMap<String, String>,
                    #[setting(nested)]
                    g: NestedExample,
                }
            });

            assert_snapshot!(pretty(container.impl_full_settings()));
        }

        #[test]
        fn includes_derived_env_with_prefix() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(env_prefix = "PREFIX_")]
                struct Example {
                    a: String,
                    #[setting(env = "B_VAR")]
                    b: i32,
                    #[setting(nested)]
                    c: NestedExample,
                    // Never read from the environment, so no key
                    d: Vec<String>,
                    #[setting(parse_env = schematic::env::split_comma)]
                    e: Vec<String>,
                    #[setting(rename = "eff")]
                    f: Option<bool>,
                    // A nested override doesn't change the child's own keys
                    #[setting(nested, env_prefix = "OVERRIDE_")]
                    g: NestedExample,
                }
            });

            assert_snapshot!(pretty(container.impl_full_settings()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn supports() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    String,
                    i32,
                    #[setting(env = "C_VAR")]
                    bool,
                    Option<String>,
                    Vec<u8>,
                    std::collections::HashMap<String, String>,
                    #[setting(nested)]
                    NestedExample,
                );
            });

            assert_snapshot!(pretty(container.impl_full_settings()));
        }
    }

    mod named_enum {
        // N/A
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn supports() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(String),
                    B(i32),
                    C(bool),
                    D(Option<String>),
                    E(Vec<u8>),
                    F(std::collections::HashMap<String, String>),
                    #[setting(nested)]
                    G(NestedExample),
                }
            });

            assert_snapshot!(pretty(container.impl_full_settings()));
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn supports() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A, B, C
                }
            });

            assert_snapshot!(pretty(container.impl_full_settings()));
        }
    }
}
