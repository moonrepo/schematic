use proc_macro2::TokenStream;
use schematic_core::container::Container;
use syn::parse_quote;

fn pretty_attrs(attrs: Vec<TokenStream>) -> String {
    attrs
        .into_iter()
        .map(|attr| attr.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

mod setting_partial {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(partial(other(attribute), and(another)))]
                    a: bool,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.partial.len(), 1);
        }

        #[test]
        fn can_repeat() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(partial(other(attribute)), partial(and(another)))]
                    #[setting(partial(last(one)))]
                    a: bool,
                }
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.partial.len(), 3);
            assert_eq!(
                pretty_attrs(field.get_partial_attributes()),
                "# [serde (skip_serializing_if = \"Option::is_none\")]\n# [other (attribute)]\n# [and (another)]\n# [last (one)]"
            );
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(partial(other(attribute), and(another)))]
                    bool,
                );
            });
            let field = container.inner.get_fields()[0];

            assert_eq!(field.args.partial.len(), 1);
        }
    }

    mod named_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(partial(other(attribute), and(another)))]
                    A {
                        field: bool
                    }
                }
            });
            let field = container.inner.get_variants()[0];

            assert_eq!(field.args.partial.len(), 1);
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(partial(other(attribute), and(another)))]
                    A(bool),
                }
            });
            let field = container.inner.get_variants()[0];

            assert_eq!(field.args.partial.len(), 1);
        }

        #[test]
        fn can_repeat() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(partial(other(attribute)))]
                    #[setting(partial(and(another)))]
                    A(bool),
                }
            });
            let variant = container.inner.get_variants()[0];

            assert_eq!(variant.args.partial.len(), 2);
            assert_eq!(
                pretty_attrs(variant.get_partial_attributes()),
                "# [other (attribute)]\n# [and (another)]"
            );
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(partial(other(attribute), and(another)))]
                    A,
                }
            });
            let field = container.inner.get_variants()[0];

            assert_eq!(field.args.partial.len(), 1);
        }
    }
}
