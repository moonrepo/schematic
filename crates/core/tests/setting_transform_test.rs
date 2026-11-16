use schematic_core::container::Container;
use syn::parse_quote;

mod setting_transform {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(transform = func_ref)]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(transform = "func_ref")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at transform")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(transform = 123)]
                    a: String,
                }
            });
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(transform = func_ref)]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(transform = "func_ref")]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at transform")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(transform = 123)]
                    String,
                );
            });
        }
    }

    mod named_enum {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(transform = func_ref)]
                        a: String,
                    },
                }
            });
            let field = &container.inner.get_variants()[0].settings[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(transform = "func_ref")]
                        a: String,
                    },
                }
            });
            let field = &container.inner.get_variants()[0].settings[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at transform")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(transform = 123)]
                        a: String,
                    },
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `transform` on a named variant")]
        fn errors_on_the_variant() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(transform = func_ref)]
                    A {
                        a: String,
                    },
                }
            });
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(transform = func_ref)]
                    A(String)
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(transform = "func_ref")]
                    A(String)
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.transform.is_some());
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at transform")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(transform = 123)]
                    A(String)
                }
            });
        }
    }

    mod unit_enum {
        // N/A
    }
}
