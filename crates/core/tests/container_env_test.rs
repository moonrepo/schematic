mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod container_env {
    use super::*;

    #[test]
    fn can_set_vars() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                no_var: String,
                no_var_nested: NestedConfig,
                #[setting(env = "STR")]
                a: String,
                #[setting(env = "BOOL")]
                b: bool,
                #[setting(env = "INT")]
                c: usize,
                #[setting(nested)]
                d: NestedConfig,
                #[setting(nested = CustomConfig)]
                e: CustomConfig,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    #[test]
    fn skips_derived_keys_without_a_prefix() {
        // Keys are only derived when a prefix is declared, otherwise
        // settings must opt in with an explicit `env`
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: String,
                #[setting(env = "B_VAR")]
                b: usize,
                #[setting(nested)]
                c: NestedConfig,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    #[test]
    fn skips_unnamed_settings_for_prefix() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "PREFIX_")]
            struct Example(
                String,
                #[setting(env = "B_VAR")]
                usize,
            );
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    #[test]
    fn skips_unsupported_types_for_prefix() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "PREFIX_")]
            struct Example {
                a: String,
                b: Vec<String>,
                c: Box<String>,
                d: Option<String>,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    #[test]
    fn can_set_prefix() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "PREFIX_")]
            struct Example {
                #[setting(env = "OVERRIDE")]
                a: String,
                b: bool,
                c: usize,
                #[setting(nested)]
                d: NestedConfig,
                #[setting(nested = CustomConfig)]
                e: CustomConfig,
                #[setting(nested, env_prefix = "NESTED_")]
                f: NestedConfig,
                #[setting(nested = CustomConfig, env_prefix = "NESTED_")]
                g: CustomConfig,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    mod named_enum {
        use super::*;

        // A named variant's settings derive keys with the enum's prefix, as a
        // struct's do, and read them when it's finalized, so the environment
        // layer holds no variant
        #[test]
        fn can_set_prefix() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(env_prefix = "PREFIX_")]
                #[serde(rename_all_fields = "camelCase")]
                enum Example {
                    #[setting(default)]
                    Off,
                    Plain(String),
                    A {
                        #[setting(env = "OVERRIDE")]
                        a: String,
                        long_name: bool,
                        #[setting(rename = "renamed")]
                        c: usize,
                        d: Vec<String>,
                        #[setting(nested)]
                        e: NestedConfig,
                    },
                    B {
                        #[setting(default = 3)]
                        c: usize,
                        #[setting(required)]
                        f: Option<String>,
                    },
                }
            });

            assert!(container.impl_partial_env_values().is_empty());
            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `env_prefix` on a `Config` enum without named variants"
        )]
        fn errors_without_named_variants() {
            Container::from(parse_quote! {
                #[derive(Config)]
                #[config(env_prefix = "PREFIX_")]
                enum Example {
                    A(String),
                    B,
                }
            })
            .validate_attributes();
        }

        #[test]
        #[should_panic(expected = "Attribute `env_prefix` cannot be empty.")]
        fn errors_if_empty() {
            Container::from(parse_quote! {
                #[derive(Config)]
                #[config(env_prefix = "")]
                enum Example {
                    A { a: String },
                }
            })
            .validate_attributes();
        }
    }
}
