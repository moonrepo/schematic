use schematic_core::container::Container;
use syn::parse_quote;

mod setting_required {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: Option<String>,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required = 123)]
                    a: Option<String>,
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with non-optional settings.")]
        fn errors_no_option() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: String,
                }
            });
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required)]
                    Option<String>,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required = 123)]
                    Option<String>,
                );
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with non-optional settings.")]
        fn errors_no_option() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required)]
                    String,
                );
            });
        }
    }

    mod named_enum {
        // N/A
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required)]
                    A(Option<String>),
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required = 123)]
                    A(Option<String>),
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with non-optional settings.")]
        fn errors_no_option() {
            Container::from(parse_quote! {
                enum Example {
                    #[setting(required)]
                    A(String),
                }
            });
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Cannot use `required` with unit variants.")]
        fn errors_for_unit() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required)]
                    A,
                }
            });
        }
    }
}
