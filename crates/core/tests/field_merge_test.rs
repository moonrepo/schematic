use schematic_core::container::Container;
use syn::parse_quote;

mod field_merge {
    use super::*;

    #[test]
    fn accepts_func_ref() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(merge = func_ref)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.merge.is_some());
    }

    #[test]
    fn accepts_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(merge = "func_ref")]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.merge.is_some());
    }

    #[test]
    #[should_panic(expected = "UnexpectedType")]
    fn errors_invalid_type() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(merge = 123)]
                a: String,
            }
        });
    }
}
