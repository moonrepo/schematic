use schematic_core::container::Container;
use starbase_sandbox::assert_debug_snapshot;
use syn::parse_quote;

mod field_default {
    use super::*;

    #[test]
    fn supports_bool() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = true)]
                a: bool,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }

    #[test]
    fn supports_number() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = 100)]
                a: usize,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }

    #[test]
    fn supports_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = "abc")]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }

    #[test]
    fn supports_array() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = ["a".into(), "b".into(), "c".into()])]
                a: [String; 3],
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }

    #[test]
    fn supports_vec() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = vec!["a", "b", "c"])]
                a: Vec<String>,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }

    #[test]
    fn supports_tuple() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = (10, -10, 0))]
                a: (usize, isize, u8),
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.default.is_some());
        assert_debug_snapshot!(field.args.default.as_ref().unwrap());
    }
}
