mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod field_env {
    use super::*;

    #[test]
    #[should_panic(expected = "Wrapper types cannot be used with `env`.")]
    fn errors_if_using_wrappers() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env = "KEY")]
                a: Arc<String>,
            }
        })
        .impl_partial_env_values();
    }

    #[test]
    #[should_panic(expected = "Collection types cannot be used with `env`.")]
    fn errors_if_using_collections() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env = "KEY")]
                a: Vec<String>,
            }
        })
        .impl_partial_env_values();
    }

    mod named_struct {
        use super::*;

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(env = "KEY")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.env.as_ref().unwrap(), "KEY");
        }

        #[test]
        #[should_panic(expected = "Attribute `env` cannot be empty.")]
        fn errors_if_empty() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(env = "")]
                    a: String,
                }
            })
            .impl_partial_env_values();
        }

        #[test]
        fn supports_different_types() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    no_env: String,
                    #[setting(env = "A")]
                    a: String,
                    #[setting(env = "B")]
                    b: usize,
                    #[setting(env = "C")]
                    c: bool,
                    #[setting(env = "D")]
                    d: f32,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_env_values()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: NestedConfig,
                    #[setting(nested = CustomConfig)]
                    b: CustomConfig,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_env_values()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(env = "KEY")]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.env.as_ref().unwrap(), "KEY");
        }

        #[test]
        #[should_panic(expected = "Attribute `env` cannot be empty.")]
        fn errors_if_empty() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(env = "")]
                    String,
                );
            })
            .impl_partial_env_values();
        }

        #[test]
        fn supports_different_types() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    String,
                    #[setting(env = "A")]
                    String,
                    #[setting(env = "B")]
                    usize,
                    #[setting(env = "C")]
                    bool,
                    #[setting(env = "D")]
                    f32,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_env_values()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested = CustomConfig)]
                    CustomConfig,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_env_values()));
        }
    }
}

mod field_env_prefix {
    use super::*;

    #[test]
    fn accepts_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env_prefix = "KEY", nested)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert_eq!(field.args.env_prefix.as_ref().unwrap(), "KEY");
    }

    #[test]
    #[should_panic(expected = "Attribute `env_prefix` cannot be empty.")]
    fn errors_if_empty() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env_prefix = "", nested)]
                a: String,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `env_prefix` without `nested`.")]
    fn errors_if_not_nested() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env_prefix = "KEY")]
                a: String,
            }
        });
    }
}

mod field_parse_env {
    use super::*;

    #[test]
    fn accepts_func_ref() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env = "KEY", parse_env = func_ref)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.parse_env.is_some());
    }

    #[test]
    fn accepts_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env = "KEY", parse_env = "func_ref")]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.parse_env.is_some());
    }

    #[test]
    #[should_panic(expected = "UnexpectedType")]
    fn errors_invalid_type() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(env = "KEY", parse_env = 123)]
                a: String,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `parse_env` without `env`.")]
    fn errors_without_env() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(parse_env = func_ref)]
                a: String,
            }
        });
    }
}
