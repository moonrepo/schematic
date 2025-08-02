mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod field_extend {
    use super::*;

    #[test]
    fn can_set_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: String,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    fn can_set_opt_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: Option<String>,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    fn can_set_vec_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: Vec<String>,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    fn can_set_opt_vec_string() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: Option<Vec<String>>,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    fn can_set_type() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: ExtendsFrom,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    fn can_set_opt_type() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: Option<ExtendsFrom>,
            }
        });
        let field = container.inner.get_fields()[0];

        assert!(field.args.extend);
        assert_snapshot!(pretty(container.impl_partial_extends_from()));
    }

    #[test]
    #[should_panic(expected = "Only 1 setting may use `extend`, found: a, b")]
    fn errors_multiple_extends() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: String,
                #[setting(extend)]
                b: String,
            }
        })
        .impl_partial_extends_from();
    }

    #[test]
    #[should_panic(
        expected = "Only `String`, `Vec<String>`, or `schematic::ExtendsFrom` are supported"
    )]
    fn errors_invalid_type() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(extend)]
                a: bool,
            }
        })
        .impl_partial_extends_from();
    }

    #[test]
    #[should_panic(expected = "Only named structs can use `extend` settings.")]
    fn errors_when_used_in_unnamed_struct() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example(
                #[setting(extend)]
                String,
            );
        })
        .impl_partial_extends_from();
    }
}
