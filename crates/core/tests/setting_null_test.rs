use schematic_core::container::Container;
use syn::parse_quote;

// Only applies to unit enums!
mod setting_null {
    use super::*;

    // mod named_enum {
    //     use super::*;

    //     #[test]
    //     #[should_panic(expected = "Can only use `null` with unit variants.")]
    //     fn errors_for_named() {
    //         Container::from(parse_quote! {
    //             #[derive(Config)]
    //             enum Example {
    //                 #[setting(null)]
    //                 A {}
    //             }
    //         });
    //     }
    // }

    mod unnamed_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Can only use `null` with unit variants.")]
        fn errors_for_named() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(null)]
                    A(String)
                }
            });
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn can_set() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(null)]
                    A,
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.null);
        }
    }
}
