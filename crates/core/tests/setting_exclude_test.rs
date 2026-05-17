use schematic_core::container::Container;
use syn::parse_quote;

mod setting_exclude {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(exclude)]
                    a: bool,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.exclude);
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(exclude)]
                    bool,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.exclude);
        }
    }

    mod named_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(exclude)]
                    A {
                        field: bool
                    }
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.exclude);
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(exclude)]
                    A(bool),
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.exclude);
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(exclude)]
                    A,
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.exclude);
        }
    }
}
