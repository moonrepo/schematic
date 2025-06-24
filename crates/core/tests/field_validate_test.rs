use schematic_core::container::Container;
use syn::parse_quote;

mod field_validate {
    use super::*;

    #[test]
    fn accepts_func_ref() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(validate = func_ref)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.validate.is_some());
    }

    #[test]
    fn accepts_curried_func() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(validate = func_call())]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.validate.is_some());
    }

    #[test]
    #[should_panic(expected = "UnexpectedType")]
    fn errors_invalid_type() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(validate = 123)]
                a: String,
            }
        });
    }
}
