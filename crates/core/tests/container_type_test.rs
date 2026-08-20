mod utils;

use quote::ToTokens;
use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod partial_type {
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
                    d: Option<String>,
                    e: Vec<String>,
                    f: Option<HashMap<u8, String>>,
                    g: Box<String>,
                    h: Arc<Vec<u8>>,
                    i: std::collections::BTreeSet<u8>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
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
                    e: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    f: HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    g: Option<BTreeSet<NestedConfig>>,
                    #[setting(nested)]
                    h: config::NestedConfig,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn strips_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: Box<String>,
                    b: Arc<usize>,
                    c: Rc<bool>,
                    d: Option<Arc<String>>,
                    e: Arc<Vec<String>>,
                    f: Vec<Arc<String>>,
                    g: Box<Arc<usize>>,
                    h: HashMap<String, Box<usize>>,
                    // Unsized values keep their wrapper
                    i: Box<str>,
                    j: Arc<str>,
                    k: Box<[u8]>,
                    // The outer `Option` is found after stripping
                    l: Box<Option<String>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn strips_wrappers_around_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Box<NestedConfig>,
                    #[setting(nested)]
                    b: Arc<NestedConfig>,
                    #[setting(nested)]
                    c: Option<Arc<NestedConfig>>,
                    #[setting(nested)]
                    d: Arc<Vec<NestedConfig>>,
                    #[setting(nested)]
                    e: Vec<Arc<NestedConfig>>,
                    #[setting(nested = CustomConfig)]
                    f: HashMap<String, Box<CustomConfig>>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn supports_attributes() {
            let container = Container::from(parse_quote! {
                /// Container docs.
                #[derive(Config)]
                #[config(allow_unknown_fields, partial(derive(Other)))]
                #[serde(rename_all = "kebab-case")]
                struct Example {
                    /// Field docs.
                    #[setting(rename = "aa", partial(serde(default)))]
                    a: bool,
                    #[serde(alias = "bb", flatten)]
                    b: HashMap<String, String>,
                    #[setting(skip)]
                    c: usize,
                    #[serde(skip_serializing, skip_deserializing)]
                    d: usize,
                    #[setting(skip_serializing_if = "is_zero")]
                    e: usize,
                    #[allow(dead_code)]
                    #[deprecated]
                    #[cfg(feature = "example")]
                    #[serde(rename(deserialize = "ff"))]
                    f: usize,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn inherits_visibility() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                pub(crate) struct Example {
                    pub a: bool,
                    pub(super) b: bool,
                    c: bool,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn strips_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    Box<String>,
                    Option<Arc<usize>>,
                    Vec<Rc<bool>>,
                    #[setting(nested)]
                    Arc<NestedConfig>,
                    // Unsized values keep their wrapper
                    Box<str>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                pub struct Example(
                    pub bool,
                    Option<String>,
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested)]
                    Option<Vec<NestedConfig>>,
                    /// Docs.
                    #[setting(partial(serde(default)))]
                    Box<usize>,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                pub enum Example {
                    /// Docs.
                    #[setting(rename = "aa")]
                    A(bool),
                    #[serde(alias = "bb", skip)]
                    B(usize, String),
                    #[setting(nested)]
                    C(NestedConfig),
                    #[setting(nested)]
                    D(Option<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    E(Vec<CustomConfig>),
                    #[setting(untagged, partial(serde(default)))]
                    F(String),
                    #[serde(other)]
                    G,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn strips_wrappers() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(Box<String>),
                    B(Option<Arc<usize>>),
                    C(Vec<Rc<bool>>),
                    #[setting(nested)]
                    D(Arc<NestedConfig>),
                    #[setting(nested)]
                    E(Option<Box<NestedConfig>>),
                    // Unsized values keep their wrapper
                    F(Box<str>),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn supports_tagged() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type", content = "value")]
                enum Example {
                    A(bool),
                    B(usize),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }

        #[test]
        fn supports_untagged() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(untagged)]
                enum Example {
                    A(bool),
                    B(usize),
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    /// Docs.
                    A,
                    #[setting(rename = "bb")]
                    B,
                    #[serde(skip_deserializing)]
                    C,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_type()));
        }
    }
}

mod partial_type_default {
    use super::*;

    #[test]
    fn skips_structs() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: bool,
            }
        });

        assert!(container.impl_partial_type_default().is_empty());
    }

    #[test]
    fn uses_marked_variant() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A(bool),
                #[setting(default)]
                B(usize, String),
                C,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_type_default()));
    }

    #[test]
    fn falls_back_to_first_variant() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A(bool),
                B(usize),
            }
        });

        assert_snapshot!(pretty(container.impl_partial_type_default()));
    }

    #[test]
    fn supports_unit_variants() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A,
                #[setting(default)]
                B,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_type_default()));
    }

    #[test]
    #[should_panic(expected = "Enums must have at least 1 variant.")]
    fn errors_for_empty_enum() {
        Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {}
        })
        .impl_partial_type_default();
    }
}

mod partial_type_deserialize {
    use super::*;

    #[test]
    fn skips_tagged_enums() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A(bool),
            }
        });

        assert!(container.impl_partial_type_deserialize().is_empty());
    }

    #[test]
    fn supports_untagged_enums() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(untagged)]
            enum Example {
                A(bool),
                #[setting(rename = "bb")]
                B(usize, String),
                #[setting(nested)]
                C(NestedConfig),
                #[setting(nested)]
                D(Option<Vec<NestedConfig>>),
                E,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_type_deserialize()));
    }
}

mod schematic {
    use super::*;

    #[test]
    fn implements_both_types() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty(container.impl_schematic()));
    }
}

mod to_tokens {
    use super::*;

    #[test]
    fn named_struct() {
        let container = Container::from(parse_quote! {
            /// Docs.
            #[derive(Config)]
            #[config(env_prefix = "EXAMPLE_")]
            pub struct Example {
                #[setting(default = true, env = "ENABLED")]
                pub enabled: bool,
                #[setting(default = "abc", validate = validate_name)]
                pub name: String,
                #[setting(required)]
                pub port: Option<u16>,
                #[setting(nested)]
                pub nested: NestedConfig,
                #[setting(nested, merge = merge::append_vec)]
                pub list: Vec<NestedConfig>,
                #[setting(extend)]
                pub extends: Vec<String>,
                #[setting(transform = transform_map)]
                pub map: HashMap<String, usize>,
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn unnamed_struct() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            pub struct Example(
                #[setting(default = 10)]
                pub usize,
                #[setting(nested)]
                pub Option<NestedConfig>,
            );
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn unnamed_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(untagged)]
            pub enum Example {
                #[setting(default)]
                Bool(bool),
                Number(usize),
                #[setting(nested)]
                Nested(NestedConfig),
                #[setting(nested, merge = merge::append_vec, transform = transform_list)]
                List(Vec<NestedConfig>),
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn unit_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(rename_all = "kebab-case")]
            pub enum Example {
                A,
                #[setting(default)]
                B,
                C,
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }
}

// Every emitted item has to carry the type arguments through: the partial
// declaration, its `Default`/`Deserialize`, `PartialConfig`, `Config`, and
// both `Schematic` impls.
mod generics {
    use super::*;

    #[test]
    fn named_struct() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example<T> {
                inner: T,
                label: String,
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn unnamed_struct() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example<T, U>(T, U);
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn unnamed_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example<T> {
                Value(T),
                #[setting(default)]
                Nothing,
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn supports_bounds_and_where_clauses() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example<T: Clone>
            where
                T: Default,
            {
                inner: T,
            }
        });

        assert_snapshot!(pretty(container.to_token_stream()));
    }

    #[test]
    fn untagged_enum_deserialize_carries_generics() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(untagged)]
            enum Example<T> {
                Value(T),
                #[setting(default)]
                Nothing,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_type_deserialize()));
    }
}
