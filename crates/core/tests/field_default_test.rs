mod utils;

use schematic_core::container::Container;
use starbase_sandbox::{assert_debug_snapshot, assert_snapshot};
use std::collections::BTreeMap;
use syn::parse_quote;
use utils::pretty;

mod field_default {
    use super::*;

    #[test]
    fn handles_collections() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: HashMap<String, String>,
                b: Vec<u8>,
                c: BTreeSet<bool>,
                d: CustomVec<usize>,
                e: UnknownCollection<isize>,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    #[test]
    fn handles_layers() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: Option<String>,
                b: Arc<u8>,
                c: Box<bool>,
                d: Rc<Option<usize>>,
                e: Arc<Vec<Option<isize>>>,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    #[test]
    fn supports_handler_func() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = handler)]
                a: String,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    mod named_struct {
        use super::*;

        #[test]
        fn supports_types() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    no_default: bool,
                    #[setting(default = true)]
                    a: bool,
                    #[setting(default = 100)]
                    b: usize,
                    #[setting(default = "abc")]
                    c: String,
                    #[setting(default = ["a".into(), "b".into(), "c".into()])]
                    d: [String; 3],
                    #[setting(default = vec!["a", "b", "c"])]
                    e: Vec<String>,
                    #[setting(default = (10, -10, 0))]
                    f: (usize, isize, u8),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));

            for field in container.inner.get_fields() {
                if field.ident.as_ref().is_some_and(|id| id != "no_default") {
                    assert!(field.args.default.is_some());
                }
            }

            let defaults = container
                .inner
                .get_fields()
                .into_iter()
                .map(|field| (field.ident.as_ref().unwrap(), field.args.default.as_ref()))
                .collect::<BTreeMap<_, _>>();

            assert_debug_snapshot!(defaults);
        }

        #[test]
        fn renders_nothing_if_all_option_wrapped() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: Option<String>,
                    b: Option<Vec<u8>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn supports_types() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    bool,
                    #[setting(default = true)]
                    bool,
                    #[setting(default = 100)]
                    usize,
                    #[setting(default = "abc")]
                    String,
                    #[setting(default = ["a".into(), "b".into(), "c".into()])]
                    [String; 3],
                    #[setting(default = vec!["a", "b", "c"])]
                    Vec<String>,
                    #[setting(default = (10, -10, 0))]
                    (usize, isize, u8),
                );
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));

            for field in container.inner.get_fields() {
                if field.index != 0 {
                    assert!(field.args.default.is_some());
                }
            }

            let defaults = container
                .inner
                .get_fields()
                .into_iter()
                .map(|field| (field.index, field.args.default.as_ref()))
                .collect::<BTreeMap<_, _>>();

            assert_debug_snapshot!(defaults);
        }

        #[test]
        fn renders_nothing_if_all_option_wrapped() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    Option<String>,
                    Option<Vec<u8>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));
        }
    }
}
