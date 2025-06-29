mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod variant_default {
    use super::*;

    #[test]
    #[should_panic(expected = "Only 1 variant may be marked as default.")]
    fn errors_if_multiple_defaults() {
        Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                #[setting(default)]
                Foo,
                #[setting(default)]
                Bar,
                #[setting(default)]
                Baz,
            }
        })
        .impl_partial_default_values();
    }

    #[test]
    #[should_panic(expected = "Enums with named fields are not supported!")]
    fn errors_for_named_enum() {
        Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                Foo {},
                #[setting(default)]
                Bar {},
                Baz {},
            }
        })
        .impl_partial_default_values();
    }

    #[test]
    fn unit_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                Foo,
                #[setting(default)]
                Bar,
                Baz,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));

        let variants = container.inner.get_variants();

        assert!(variants[1].args.default);
    }

    #[test]
    fn unnamed_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                Foo(usize),
                #[setting(default)]
                Bar(String, u8),
                Baz(bool, String, isize),
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));

        let variants = container.inner.get_variants();

        assert!(variants[1].args.default);
    }
}
