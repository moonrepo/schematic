mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod container_finalize {
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
                    #[setting(transform = transform_string)]
                    c: String,
                    d: i16,
                    e: Option<String>,
                    #[setting(transform = "transform_vec")]
                    f: Vec<String>,
                    g: Option<HashMap<u8, String>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: NestedConfig,
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    b: CustomConfig,
                    #[setting(nested)]
                    c: Option<NestedConfig>,
                    #[setting(nested = CustomConfig, transform = "transform_config")]
                    d: Arc<CustomConfig>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    b: HashMap<String, CustomConfig>,
                    #[setting(nested, transform = transform_config)]
                    c: Option<BTreeSet<NestedConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
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
                    #[setting(transform = transform_string)]
                    String,
                    i16,
                    Option<String>,
                    #[setting(transform = "transform_vec")]
                    Vec<String>,
                    Option<HashMap<u8, String>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    CustomConfig,
                    #[setting(nested)]
                    Option<NestedConfig>,
                    #[setting(nested = CustomConfig, transform = "transform_config")]
                    Arc<CustomConfig>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    HashMap<String, CustomConfig>,
                    #[setting(nested, transform = transform_config)]
                    Option<BTreeSet<NestedConfig>>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
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
                    #[setting(transform = transform_string)]
                    C(String),
                    D(i16),
                    E(Option<String>),
                    #[setting(transform = "transform_vec")]
                    F(Vec<String>),
                    G(Option<HashMap<u8, String>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig),
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    B(CustomConfig),
                    #[setting(nested)]
                    C(Option<NestedConfig>),
                    #[setting(nested = CustomConfig, transform = "transform_config")]
                    D(Arc<CustomConfig>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }

        #[test]
        fn supports_nested_collections() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig, transform = transform_config)]
                    B(HashMap<String, CustomConfig>),
                    #[setting(nested, transform = transform_config)]
                    C(Option<BTreeSet<NestedConfig>>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_finalize()));
        }
    }

    mod unit_enum {
        // N/A
    }
}
