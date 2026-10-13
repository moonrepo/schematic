use schematic_core::container::Container;
use starbase_sandbox::assert_debug_snapshot;
use syn::parse_quote;

mod container_partial {
    use super::*;

    #[test]
    fn can_set() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(partial(derive(Other), serde(another)))]
            struct Example {}
        });

        assert_eq!(container.args.partial.len(), 1);
        assert_debug_snapshot!(&container.args.partial[0]);
    }
}
