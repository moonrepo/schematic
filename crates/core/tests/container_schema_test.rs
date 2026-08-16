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

mod schematic {
    use super::*;

    #[test]
    fn implements_both_types() {
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
                    // Derived env keys are not statically known
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
        fn supports_many_values() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    String,
                    /// Item docs.
                    usize,
                    #[setting(nested)]
                    NestedConfig,
                );
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn supports_external() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    /// Docs.
                    A(String),
                    #[setting(default)]
                    B(usize, bool),
                    #[setting(nested)]
                    C(NestedConfig),
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
                    A(String),
                    #[setting(nested)]
                    B(NestedConfig),
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
                    A(String),
                    #[setting(nested)]
                    B(NestedConfig),
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
                    A(String),
                    #[setting(nested)]
                    B(NestedConfig),
                }
            });

            assert_snapshot!(pretty_body(container.impl_schematic_type()));
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
    }
}
