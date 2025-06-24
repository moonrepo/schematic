use schematic_core::container::Container;
use syn::parse_quote;

mod field_env {
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
        });
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
