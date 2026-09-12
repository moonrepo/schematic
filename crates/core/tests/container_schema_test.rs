mod utils;

use proc_macro2::TokenStream;
use quote::quote;
use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

// `impl_schematic_type` returns a method body, so wrap it to be parsable
fn pretty_body(tokens: TokenStream) -> String {
    pretty(quote! {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            #tokens
        }
    })
}

// The trait implementations for both the full and partial types
mod schematic {
    use super::*;

    #[test]
    fn named_struct() {
        let container = Container::from(parse_quote! {
            /// Container docs.
            #[derive(Config)]
            #[config(rename = "Renamed")]
            #[deprecated = "Use something else."]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty(container.impl_schematic()));
    }

    #[test]
    fn unnamed_struct() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example(bool, usize);
        });

        assert_snapshot!(pretty(container.impl_schematic()));
    }

    #[test]
    fn unnamed_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[serde(rename = "Renamed")]
            enum Example {
                A(bool),
            }
        });

        assert_snapshot!(pretty(container.impl_schematic()));
    }

    #[test]
    fn unit_enum() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A,
                B,
            }
        });

        assert_snapshot!(pretty(container.impl_schematic()));
    }
}

mod schema_type {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn supports_standard() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: bool,
                    /// Some docs.
                    b: usize,
                    #[setting(default = "abc")]
                    c: String,
                    #[setting(default = 10)]
                    d: usize,
                    #[setting(default = 1.5)]
                    e: f32,
                    #[setting(default = true)]
                    f: bool,
                    g: Option<String>,
                    h: Vec<String>,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_metadata() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(env_prefix = "APP_")]
                struct Example {
                    /// Docs.
                    #[deprecated = "Gone soon."]
                    #[setting(rename = "aa", env = "A_VAR")]
                    a: String,
                    #[serde(alias = "bee", alias = "bb", flatten)]
                    b: HashMap<String, String>,
                    #[setting(skip)]
                    c: usize,
                    // Derived from the container prefix
                    d: usize,
                    #[setting(exclude)]
                    e: usize,
                    #[setting(nested)]
                    f: NestedConfig,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: NestedConfig,
                    #[setting(nested)]
                    b: Option<NestedConfig>,
                    #[setting(nested)]
                    c: Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    d: HashMap<String, CustomConfig>,
                    #[setting(nested)]
                    e: Box<NestedConfig>,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_empty() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {}
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn supports_single_value() {
            let container = Container::from(parse_quote! {
                /// Docs.
                #[derive(Config)]
                struct Example(String);
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_single_nested_value() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(#[setting(nested)] NestedConfig);
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_single_nested_collection() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(#[setting(nested)] Vec<NestedConfig>);
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_many_values() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    String,
                    /// Item docs.
                    Vec<usize>,
                    #[setting(nested)]
                    NestedConfig,
                    #[setting(nested)]
                    Vec<NestedConfig>,
                    #[setting(nested = CustomConfig)]
                    Option<CustomConfig>,
                    #[setting(exclude)]
                    bool,
                );
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_empty() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example();
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }
    }

    // Every tagging format uses the same variants, so that the
    // output can be compared across them
    mod unnamed_enum {
        use super::*;

        #[test]
        fn supports_external() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    /// Docs.
                    Text(String),
                    List(Vec<String>),
                    #[setting(default)]
                    Pair(usize, bool),
                    #[setting(nested)]
                    Inner(NestedConfig),
                    #[setting(nested)]
                    InnerOpt(Option<NestedConfig>),
                    #[setting(nested)]
                    InnerList(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    InnerMap(HashMap<String, CustomConfig>),
                    Unit,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_untagged() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(untagged)]
                enum Example {
                    /// Docs.
                    Text(String),
                    List(Vec<String>),
                    #[setting(default)]
                    Pair(usize, bool),
                    #[setting(nested)]
                    Inner(NestedConfig),
                    #[setting(nested)]
                    InnerOpt(Option<NestedConfig>),
                    #[setting(nested)]
                    InnerList(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    InnerMap(HashMap<String, CustomConfig>),
                    Unit,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_internal_tag() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type")]
                enum Example {
                    /// Docs.
                    Text(String),
                    List(Vec<String>),
                    #[setting(default)]
                    Pair(usize, bool),
                    #[setting(nested)]
                    Inner(NestedConfig),
                    #[setting(nested)]
                    InnerOpt(Option<NestedConfig>),
                    #[setting(nested)]
                    InnerList(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    InnerMap(HashMap<String, CustomConfig>),
                    Unit,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_adjacent_tag() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type", content = "value")]
                enum Example {
                    /// Docs.
                    Text(String),
                    List(Vec<String>),
                    #[setting(default)]
                    Pair(usize, bool),
                    #[setting(nested)]
                    Inner(NestedConfig),
                    #[setting(nested)]
                    InnerOpt(Option<NestedConfig>),
                    #[setting(nested)]
                    InnerList(Vec<NestedConfig>),
                    #[setting(nested = CustomConfig)]
                    InnerMap(HashMap<String, CustomConfig>),
                    Unit,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_variant_untagged() {
            // A single variant can opt out of the container's tagging
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type")]
                enum Example {
                    Text(String),
                    #[setting(untagged)]
                    Inner(String),
                    #[serde(untagged)]
                    Other(String),
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_renames_and_exclusions() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(rename = "text")]
                    Text(String),
                    #[serde(rename = "number")]
                    Number(usize),
                    #[setting(exclude)]
                    Gone(bool),
                    #[setting(default)]
                    Last(bool),
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }
    }

    // All-unit enums are enumerable values, regardless of the
    // tagging format that was configured
    mod unit_enum {
        use super::*;

        #[test]
        fn supports_external() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    /// Docs.
                    A,
                    #[setting(default, rename = "bb")]
                    B,
                    #[setting(null)]
                    C,
                    #[setting(exclude)]
                    D,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_untagged() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(untagged)]
                enum Example {
                    A,
                    #[setting(default)]
                    B,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_internal_tag() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type")]
                enum Example {
                    A,
                    #[setting(default)]
                    B,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }

        #[test]
        fn supports_adjacent_tag() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type", content = "value")]
                enum Example {
                    A,
                    #[setting(default)]
                    B,
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }
    }
}
