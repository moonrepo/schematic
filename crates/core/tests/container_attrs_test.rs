use proc_macro2::TokenStream;
use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;

// Render each attribute on its own line
fn pretty_attrs(attrs: Vec<TokenStream>) -> String {
    attrs
        .into_iter()
        .map(|attr| attr.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

mod partial_serde_args {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn defaults() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn allows_unknown_fields() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(allow_unknown_fields)]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn serde_denies_unknown_fields_over_config() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(allow_unknown_fields)]
                #[serde(deny_unknown_fields)]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn inherits_serde_args() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(rename = "Renamed", rename_all = "kebab-case", expecting = "expected")]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn inherits_config_args() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(rename = "Renamed", rename_all = "kebab-case")]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn config_args_take_precedence() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[config(rename = "FromConfig")]
                #[serde(rename = "FromSerde", rename_all = "kebab-case")]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn supports_split_renames() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(
                    rename(deserialize = "De"),
                    rename_all(deserialize = "kebab-case", serialize = "camelCase"),
                )]
                struct Example {
                    a: bool,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn defaults() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(bool);
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn defaults() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A(bool),
                    B(usize),
                }
            });

            assert!(container.get_partial_serde_attribute_args().is_empty());
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

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }

        #[test]
        fn supports_tagged() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(tag = "type", content = "value", rename_all_fields = "camelCase")]
                enum Example {
                    A(bool),
                    B(usize),
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        fn defaults() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(rename_all = "kebab-case")]
                enum Example {
                    A,
                    B,
                }
            });

            assert_snapshot!(container.get_partial_serde_attribute_args().to_string());
        }
    }
}

mod partial_attrs {
    use super::*;

    #[test]
    fn renders_serde_only_by_default() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty_attrs(container.get_partial_attributes()));
    }

    #[test]
    fn skips_serde_when_empty() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            enum Example {
                A(bool),
            }
        });

        assert!(container.get_partial_attributes().is_empty());
    }

    #[test]
    fn inherits_supported_attrs() {
        let container = Container::from(parse_quote! {
            /// Some docs.
            /// More docs.
            #[derive(Config)]
            #[allow(dead_code)]
            #[expect(missing_docs)]
            #[warn(unused)]
            #[deprecated]
            #[cfg(feature = "example")]
            #[non_exhaustive]
            #[repr(C)]
            #[cfg_attr(test, derive(Other))]
            #[config(allow_unknown_fields)]
            #[serde(rename = "Renamed")]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty_attrs(container.get_partial_attributes()));
    }

    #[test]
    fn appends_partial_attrs() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(partial(derive(Other, Another), serde(rename_all = "kebab-case")))]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty_attrs(container.get_partial_attributes()));
    }

    #[test]
    fn orders_serde_then_inherited_then_partial() {
        let container = Container::from(parse_quote! {
            /// Docs.
            #[derive(Config)]
            #[config(partial(derive(Other)))]
            enum Example {
                A(bool),
            }
        });

        assert_snapshot!(pretty_attrs(container.get_partial_attributes()));
    }
}
