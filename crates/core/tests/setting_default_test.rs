mod utils;

use schematic_core::container::Container;
use starbase_sandbox::{assert_debug_snapshot, assert_snapshot};
use std::collections::BTreeMap;
use syn::parse_quote;
use utils::pretty;

mod setting_default {
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
    fn handles_layers_with_defaults() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = 10)]
                a: Box<usize>,
                #[setting(default = 10)]
                b: Arc<Option<usize>>,
                #[setting(default = vec![1, 2, 3])]
                c: Vec<usize>,
                #[setting(default = vec![1, 2, 3])]
                d: Arc<Vec<usize>>,
                #[setting(default = "abc")]
                e: Box<String>,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    #[test]
    fn handles_optional_with_defaults() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                no_default: Option<usize>,
                #[setting(default = 10)]
                a: Option<usize>,
                #[setting(default = 10)]
                b: Arc<Option<usize>>,
                #[setting(default = vec![1, 2])]
                c: Option<Vec<usize>>,
                #[setting(default = "abc")]
                d: Option<String>,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    #[test]
    fn handles_nested_layers() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: Box<NestedConfig>,
                #[setting(nested = CustomConfig)]
                b: Arc<CustomConfig>,
                #[setting(nested)]
                c: Vec<NestedConfig>,
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

    // A path that names a value rather than a function, told apart by Rust's
    // naming conventions, so that an enum variant does not compile as a call
    #[test]
    fn supports_value_paths() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = LevelFilter::Debug)]
                a: LevelFilter,
                #[setting(default = MAX)]
                b: usize,
                #[setting(default = limits::MAX)]
                c: usize,
                #[setting(default = Point::ORIGIN)]
                d: Point,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    #[test]
    fn supports_struct_literals() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = Point { x: 1, y: 2 })]
                a: Point,
                #[setting(default = Wrapper { inner: Point { x: 0, y: 0 } })]
                b: Wrapper,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_default_values()));
    }

    // An `Option` setting still gets its value wrapped, and a collection still
    // takes the whole literal
    #[test]
    fn wraps_value_paths_in_layers() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(default = LevelFilter::Debug)]
                a: Option<LevelFilter>,
                #[setting(default = LevelFilter::Debug)]
                b: Box<LevelFilter>,
                #[setting(default = vec![LevelFilter::Debug])]
                c: Vec<LevelFilter>,
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
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: NestedConfig,
                    #[setting(nested = CustomConfig)]
                    b: CustomConfig,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));
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
        fn handles_layers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    Arc<u8>,
                    Box<bool>,
                    Rc<Option<usize>>,
                    Arc<Vec<Option<isize>>>,
                    #[setting(default = 10)]
                    Box<usize>,
                    #[setting(default = 10)]
                    Arc<Option<usize>>,
                    // Unsized values keep their wrapper
                    Box<str>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));
        }

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

    mod named_enum {
        use super::*;

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
    }

    mod unnamed_enum {
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
        fn supports() {
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
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn supports() {
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
}
