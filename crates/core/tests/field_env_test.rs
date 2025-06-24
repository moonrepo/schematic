use schematic_core::container::Container;
use syn::parse_quote;

mod field_parse_env {
    use super::*;

    #[test]
    fn accepts_func_ref() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(parse_env = func_ref)]
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
                #[setting(parse_env = "func_ref")]
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
                #[setting(parse_env = 123)]
                a: String,
            }
        });
    }
}
