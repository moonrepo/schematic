mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod setting_validate {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(validate = func_ref)]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn accepts_curried_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(validate = func_call())]
                    a: String,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(validate = 123)]
                    a: String,
                }
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(validate = func_ref)]
                    a: bool,
                    #[setting(validate = func_ref)]
                    b: Option<bool>,
                    #[setting(validate = func_ref)]
                    c: Vec<String>,
                    #[setting(validate = func_ref)]
                    d: Vec<HashMap<String, usize>>,
                    #[setting(validate = func_ref)]
                    e: Option<Vec<HashMap<String, usize>>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested, validate = func_ref)]
                    a: NestedConfig,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    b: CustomConfig,
                    #[setting(nested, validate = func_ref)]
                    c: Option<NestedConfig>,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    d: Arc<CustomConfig>,
                    #[setting(nested)]
                    e: NestedConfig,
                    #[setting(nested = CustomConfig)]
                    f: CustomConfig,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested, validate = func_ref)]
                    a: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    b: HashMap<String, CustomConfig>,
                    #[setting(nested, validate = func_ref)]
                    c: Option<BTreeSet<NestedConfig>>,
                    #[setting(nested)]
                    d: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    e: HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    f: Option<BTreeSet<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(validate = func_ref)]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn accepts_curried_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(validate = func_call())]
                    String,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(validate = 123)]
                    String,
                );
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(validate = func_ref)]
                    bool,
                    #[setting(validate = func_ref)]
                    Option<bool>,
                    #[setting(validate = func_ref)]
                    Vec<String>,
                    #[setting(validate = func_ref)]
                    Vec<HashMap<String, usize>>,
                    #[setting(validate = func_ref)]
                    Option<Vec<HashMap<String, usize>>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested, validate = func_ref)]
                    NestedConfig,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    CustomConfig,
                    #[setting(nested, validate = func_ref)]
                    Option<NestedConfig>,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    Arc<CustomConfig>,
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested = CustomConfig)]
                    CustomConfig,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested, validate = func_ref)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    HashMap<String, CustomConfig>,
                    #[setting(nested, validate = func_ref)]
                    Option<BTreeSet<NestedConfig>>,
                    #[setting(nested)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    Option<BTreeSet<NestedConfig>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }
    }

    mod named_enum {
        // N/A
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn accepts_func_ref() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = func_ref)]
                    A(String),
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn accepts_curried_func() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = func_call())]
                    A(String),
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.validate.is_some());
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        #[should_panic(expected = "UnexpectedType")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = 123)]
                    A(String),
                }
            });
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = func_ref)]
                    A(bool),
                    #[setting(validate = func_ref)]
                    B(Option<bool>),
                    #[setting(validate = func_ref)]
                    C(Vec<String>),
                    #[setting(validate = func_ref)]
                    D(Vec<HashMap<String, usize>>),
                    #[setting(validate = func_ref)]
                    E(Option<Vec<HashMap<String, usize>>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested, validate = func_ref)]
                    A(NestedConfig),
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    B(CustomConfig),
                    #[setting(nested, validate = func_ref)]
                    C(Option<NestedConfig>),
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    D(Arc<CustomConfig>),
                    #[setting(nested)]
                    E(NestedConfig),
                    #[setting(nested = CustomConfig)]
                    F(CustomConfig),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested, validate = func_ref)]
                    A(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig, validate = func_ref)]
                    B(HashMap<String, CustomConfig>),
                    #[setting(nested, validate = func_ref)]
                    C(Option<BTreeSet<NestedConfig>>),
                    #[setting(nested)]
                    D(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    E(HashMap<String, CustomConfig>),
                    #[setting(nested)]
                    F(Option<BTreeSet<NestedConfig>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn supports_many_values() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = func_ref)]
                    A(bool),
                    #[setting(validate = func_ref)]
                    B(bool, usize),
                    #[setting(validate = func_ref)]
                    C(bool, usize, String),
                    #[setting(required, validate = func_ref)]
                    A(Option<bool>),
                    #[setting(required, validate = func_ref)]
                    B(Option<bool>, Option<usize>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Cannot use `validate` with unit variants.")]
        fn errors_for_unit() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(validate = func_ref)]
                    A,
                }
            });
        }
    }
}
