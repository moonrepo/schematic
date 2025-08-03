use schematic_core::container::Container;
use syn::parse_quote;

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

            assert!(field.args.partial.is_some());
        }
    }
}
