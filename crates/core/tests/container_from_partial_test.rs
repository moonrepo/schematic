mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod container_from_partial {
    use super::*;

    mod named_struct {
        use super::*;

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
                    h: Box<String>,
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
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
                    #[setting(nested)]
                    e: Box<NestedConfig>,
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    b: HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    c: Option<BTreeSet<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }

        #[test]
        fn resets_extended() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(extend)]
                    a: Vec<String>,
                    b: bool,
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }
    }

    mod unnamed_struct {
        use super::*;

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
                    Box<String>,
                );
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
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
                    #[setting(nested)]
                    Box<NestedConfig>,
                );
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    Option<BTreeSet<NestedConfig>>,
                );
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }
    }

    mod named_enum {
        // N/A
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(bool),
                    B(usize),
                    C(String),
                    D(i16),
                    E(Option<String>),
                    F(Vec<String>),
                    G(Option<HashMap<u8, String>>),
                    H(String, usize),
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
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

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    B(HashMap<String, CustomConfig>),
                    #[setting(nested)]
                    C(Option<BTreeSet<NestedConfig>>),
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn supports() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A, B, C
                }
            });

            assert_snapshot!(pretty(container.impl_full_from_partial()));
        }
    }
}
