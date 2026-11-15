mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod setting_merge {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(merge = func_ref)]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(merge = "func_ref")]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(merge = 123)]
                    a: String,
                }
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: bool,
                    b: usize,
                    c: String,
                    d: i16,
                    e: Option<String>,
                    f: Vec<String>,
                    g: Option<HashMap<u8, String>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
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
                    #[setting(nested)]
                    c: Option<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    d: Arc<CustomConfig>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Box<NestedConfig>,
                    #[setting(nested)]
                    b: Rc<NestedConfig>,
                    #[setting(nested)]
                    c: Option<Box<NestedConfig>>,
                    #[setting(nested)]
                    d: Box<Arc<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested, merge = append_vec)]
                    a: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, merge = merge_hashmap)]
                    b: HashMap<String, CustomConfig>,
                    #[setting(nested, merge = merge_btreeset)]
                    c: Option<BTreeSet<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: bool,
                    b: usize,
                    #[setting(merge = discard)]
                    c: String,
                    #[setting(merge = "preserve")]
                    d: i16,
                    e: Option<String>,
                    #[setting(merge = append_vec)]
                    f: Vec<String>,
                    #[setting(merge = merge_hashmap)]
                    g: Option<HashMap<u8, String>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(
            expected = "Nested configs do not support `merge` unless wrapped in a collection."
        )]
        fn errors_if_nested_has_merge_attr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested, merge = append_vec)]
                    a: NestedConfig,
                }
            })
            .impl_partial_merge();
        }

        #[test]
        fn replaces_collections_without_merge_attr() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Vec<NestedConfig>,
                    #[setting(nested)]
                    b: HashMap<String, NestedConfig>,
                    #[setting(nested)]
                    c: Option<Vec<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(merge = func_ref)]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(merge = "func_ref")]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(merge = 123)]
                    String,
                );
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    bool,
                    usize,
                    String,
                    i16,
                    Option<String>,
                    Vec<String>,
                    Option<HashMap<u8, String>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested = CustomConfig)]
                    CustomConfig,
                    #[setting(nested)]
                    Option<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    Arc<CustomConfig>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested, merge = append_vec)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, merge = merge_hashmap)]
                    HashMap<String, CustomConfig>,
                    #[setting(nested, merge = merge_btreeset)]
                    Option<BTreeSet<NestedConfig>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    bool,
                    usize,
                    #[setting(merge = discard)]
                    String,
                    #[setting(merge = "preserve")]
                    i16,
                    Option<String>,
                    #[setting(merge = append_vec)]
                    Vec<String>,
                    #[setting(merge = merge_hashmap)]
                    Option<HashMap<u8, String>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(
            expected = "Nested configs do not support `merge` unless wrapped in a collection."
        )]
        fn errors_if_nested_has_merge_attr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested, merge = append_vec)]
                    NestedConfig,
                );
            })
            .impl_partial_merge();
        }

        #[test]
        fn replaces_collections_without_merge_attr() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    Vec<NestedConfig>,
                    #[setting(nested)]
                    HashMap<String, NestedConfig>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }
    }

    mod named_enum {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(merge = func_ref)]
                        a: String,
                    },
                }
            });
            let field = &container.inner.get_variants()[0].settings[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(merge = "func_ref")]
                        a: String,
                    },
                }
            });
            let field = &container.inner.get_variants()[0].settings[0];

            assert!(field.args.merge.is_some());
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(merge = 123)]
                        a: String,
                    },
                }
            });
        }

        // The same variant is merged setting by setting, while another replaces it
        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        a: bool,
                    },
                    B {
                        b: usize,
                        c: String,
                        d: Option<String>,
                        e: Vec<String>,
                    },
                    C {
                        f: Option<HashMap<u8, String>>,
                        g: MergeList<String>,
                    },
                    D {},
                    E(bool),
                    F,
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(merge = a)]
                        a: bool,
                        #[setting(merge = b)]
                        b: usize,
                        c: String,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested)]
                        a: NestedConfig,
                        #[setting(nested = CustomConfig)]
                        b: CustomConfig,
                        #[setting(nested)]
                        c: Option<NestedConfig>,
                        #[setting(nested = CustomConfig)]
                        d: Arc<CustomConfig>,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested)]
                        a: Box<NestedConfig>,
                        #[setting(nested)]
                        b: Rc<NestedConfig>,
                        #[setting(nested)]
                        c: Option<Box<NestedConfig>>,
                        #[setting(nested)]
                        d: Box<Arc<NestedConfig>>,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(expected = "may only be wrapped in an outermost `Option`")]
        fn errors_for_inner_option() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested)]
                        a: Option<Option<NestedConfig>>,
                    },
                }
            })
            .impl_partial_merge();
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested, merge = append_vec)]
                        a: Vec<NestedConfig>,
                        #[setting(nested = CustomConfig, merge = merge_hashmap)]
                        b: HashMap<String, CustomConfig>,
                        #[setting(nested, merge = merge_btreeset)]
                        c: Option<BTreeSet<NestedConfig>>,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(
            expected = "Nested configs do not support `merge` unless wrapped in a collection."
        )]
        fn errors_if_nested_has_merge_attr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested, merge = append_vec)]
                        a: NestedConfig,
                    },
                }
            })
            .impl_partial_merge();
        }

        #[test]
        fn replaces_collections_without_merge_attr() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(nested)]
                        a: Vec<NestedConfig>,
                        #[setting(nested)]
                        b: HashMap<String, NestedConfig>,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(expected = "Cannot use `merge` on a named variant")]
        fn errors_on_the_variant() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = func_ref)]
                    A {
                        a: String,
                    },
                }
            });
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = func_ref)]
                    A(String),
                }
            });
            let variant = container.inner.get_variants()[0];

            assert!(variant.args.merge.is_some());
        }

        #[test]
        fn accepts_string() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = "func_ref")]
                    A(String),
                }
            });
            let variant = container.inner.get_variants()[0];

            assert!(variant.args.merge.is_some());
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = 123)]
                    A(String),
                }
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(bool),
                    B(usize),
                    C(String, i16),
                    D(Option<String>, Vec<String>),
                    E(Option<HashMap<u8, String>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = a)]
                    A(bool),
                    #[setting(merge = b)]
                    B(usize),
                    #[setting(merge = c)]
                    C(String, i16),
                    #[setting(merge = d)]
                    D(Option<String>, Vec<String>),
                    #[setting(merge = e)]
                    E(Option<HashMap<u8, String>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig),
                    #[setting(nested = CustomConfig)]
                    B(CustomConfig),
                    #[setting(nested)]
                    C(Option<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    D(Arc<CustomConfig>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_nested_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Box<NestedConfig>),
                    #[setting(nested)]
                    B(Rc<NestedConfig>),
                    #[setting(nested)]
                    C(Option<Box<NestedConfig>>),
                    #[setting(nested)]
                    D(Box<Arc<NestedConfig>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(expected = "may only be wrapped in an outermost `Option`")]
        fn errors_for_inner_option() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Option<Option<NestedConfig>>),
                }
            })
            .impl_partial_merge();
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested, merge = append_vec)]
                    A(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig, merge = merge_hashmap)]
                    B(HashMap<String, CustomConfig>),
                    #[setting(nested, merge = merge_btreeset)]
                    C(Option<BTreeSet<NestedConfig>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(
            expected = "Nested configs do not support `merge` unless wrapped in a collection."
        )]
        fn errors_if_nested_has_merge_attr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested, merge = append_vec)]
                    A(NestedConfig),
                }
            })
            .impl_partial_merge();
        }

        #[test]
        fn replaces_collections_without_merge_attr() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Vec<NestedConfig>),
                    #[setting(nested)]
                    B(HashMap<String, NestedConfig>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn supports_multiple_nested_values() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig, CustomConfig),
                    #[setting(nested)]
                    B(NestedConfig, Option<CustomConfig>),
                    // Collections are replaced in place, alongside a merge
                    #[setting(nested)]
                    C(NestedConfig, Vec<CustomConfig>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Cannot use `merge` with unit variants.")]
        fn errors_for_unit() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(merge = func_ref)]
                    A,
                }
            });
        }
    }

    mod merge_list {
        use super::*;

        #[test]
        fn infers_for_fields() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: MergeList<String>,
                    b: Option<MergeList<String>>,
                    c: Box<MergeList<String>>,
                    d: schematic::MergeList<Vec<String>>,
                    // A collection of them is replaced, as is an explicit `merge`
                    e: Vec<MergeList<String>>,
                    #[setting(merge = schematic::merge::replace)]
                    f: MergeList<String>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        fn infers_for_variant_values() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(MergeList<String>),
                    B(MergeList<String>, bool),
                    C(Option<MergeList<String>>),
                    #[setting(merge = schematic::merge::replace)]
                    D(MergeList<String>),
                    E(Vec<MergeList<String>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_merge()));
        }

        #[test]
        #[should_panic(expected = "A `MergeList` can't hold nested configs")]
        fn errors_for_nested_items() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: MergeList<NestedConfig>,
                }
            });
        }
    }
}
