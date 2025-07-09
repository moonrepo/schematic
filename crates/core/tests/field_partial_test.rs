use schematic_core::container::Container;
use starbase_sandbox::assert_debug_snapshot;
use syn::parse_quote;

mod field_partial {
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
        assert_debug_snapshot!(field.args.partial.as_ref().unwrap());
    }
}
