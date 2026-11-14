use quote::format_ident;
use schematic_core::container::Container;
use syn::parse_quote;

mod setting_nested {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn word() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: NestedConfig,
                }
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_true() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = true)]
                    a: NestedConfig,
                }
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_false() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = false)]
                    a: NestedConfig,
                }
            });
            let fields = container.inner.get_fields();

            assert!(!fields[0].value.nested);
            assert!(fields[0].value.nested_ident.is_none());
        }

        #[test]
        fn explicit_ident() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = NestedConfig)]
                    a: NestedConfig,
                }
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn detect_in_vec() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Vec<NestedConfig>,
                }
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        #[should_panic(expected = "UnexpectedType(\"paren\")")]
        fn panics_invalid_expr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = (1 + 1))]
                    a: NestedConfig,
                }
            });
        }

        // #[test]
        // #[should_panic(
        //     expected = "Unable to extract the nested configuration identifier from `Vec<Option<Box<NestedConfig>>>`. Try explicitly passing the identifier with `nested = ConfigName`."
        // )]
        // fn panics_cant_find_ident() {
        //     Container::from(parse_quote! {
        //         #[derive(Config)]
        //         struct Example {
        //             #[setting(nested)]
        //             a: Vec<Option<Box<NestedConfig>>>,
        //         }
        //     });
        // }

        #[test]
        #[should_panic(
            expected = "Too many segments for `sub::NestedConfig`, only a single identifier is allowed."
        )]
        fn panics_too_many_segments() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = sub::NestedConfig)]
                    a: NestedConfig,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Nested configuration identifier `OtherConfig` does not exist within `Vec<NestedConfig>`."
        )]
        fn panics_cant_find_custom_ident() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested = OtherConfig)]
                    a: Vec<NestedConfig>,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Nested configurations must be a `Config` type, received `bool`."
        )]
        fn panics_for_primitives() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: bool,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Nested configurations must be a `Config` type, received `Vec<String>`."
        )]
        fn panics_for_primitives_in_collections() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    a: Vec<String>,
                }
            });
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn word() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    NestedConfig,
                );
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_true() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = true)]
                    NestedConfig,
                );
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_false() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = false)]
                    NestedConfig,
                );
            });
            let fields = container.inner.get_fields();

            assert!(!fields[0].value.nested);
            assert!(fields[0].value.nested_ident.is_none());
        }

        #[test]
        fn explicit_ident() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = NestedConfig)]
                    NestedConfig,
                );
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn detect_in_vec() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested)]
                    Vec<NestedConfig>,
                );
            });
            let fields = container.inner.get_fields();

            assert!(fields[0].value.nested);
            assert_eq!(
                fields[0].value.nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        #[should_panic(expected = "UnexpectedType(\"paren\")")]
        fn panics_invalid_expr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = (1 + 1))]
                    NestedConfig,
                );
            });
        }

        // #[test]
        // #[should_panic(
        //     expected = "Unable to extract the nested configuration identifier from `Vec<Option<Box<NestedConfig>>>`. Try explicitly passing the identifier with `nested = ConfigName`."
        // )]
        // fn panics_cant_find_ident() {
        //     Container::from(parse_quote! {
        //         #[derive(Config)]
        //         struct Example(
        //             #[setting(nested)]
        //             Vec<Option<Box<NestedConfig>>>,
        //         );
        //     });
        // }

        #[test]
        #[should_panic(
            expected = "Too many segments for `sub::NestedConfig`, only a single identifier is allowed."
        )]
        fn panics_too_many_segments() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = sub::NestedConfig)]
                    NestedConfig,
                );
            });
        }

        #[test]
        #[should_panic(
            expected = "Nested configuration identifier `OtherConfig` does not exist within `Vec<NestedConfig>`."
        )]
        fn panics_cant_find_custom_ident() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(nested = OtherConfig)]
                    Vec<NestedConfig>,
                );
            });
        }
    }

    mod named_enum {
        // N/A
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn word() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig),
                }
            });
            let variants = container.inner.get_variants();

            assert!(variants[0].values[0].nested);
            assert_eq!(
                variants[0].values[0].nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_true() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested = true)]
                    A(NestedConfig),
                }
            });
            let variants = container.inner.get_variants();

            assert!(variants[0].values[0].nested);
            assert_eq!(
                variants[0].values[0].nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn bool_false() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested = false)]
                    A(NestedConfig),
                }
            });
            let variants = container.inner.get_variants();

            assert!(!variants[0].values[0].nested);
            assert!(variants[0].values[0].nested_ident.is_none());
        }

        #[test]
        fn explicit_ident() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested = NestedConfig)]
                    A(NestedConfig),
                }
            });
            let variants = container.inner.get_variants();

            assert!(variants[0].values[0].nested);
            assert_eq!(
                variants[0].values[0].nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        fn detect_in_vec() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(Vec<NestedConfig>),
                }
            });
            let variants = container.inner.get_variants();

            assert!(variants[0].values[0].nested);
            assert_eq!(
                variants[0].values[0].nested_ident.as_ref().unwrap(),
                &format_ident!("NestedConfig")
            );
        }

        #[test]
        #[should_panic(expected = "UnexpectedType(\"paren\")")]
        fn panics_invalid_expr() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested = (1 + 1))]
                    A(NestedConfig),
                }
            });
        }

        #[test]
        fn supports_multiple_items() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig, CustomConfig),
                }
            });

            assert_eq!(container.inner.get_variants()[0].values.len(), 2);
        }

        #[test]
        #[should_panic(
            expected = "Nested configurations must be a `Config` type, received `bool`."
        )]
        fn panics_if_any_item_is_not_a_config() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A(NestedConfig, bool),
                }
            });
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Cannot use `nested` with unit variants.")]
        fn errors_for_unit() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nested)]
                    A,
                }
            });
        }
    }
}

mod audit_fixes {
    use quote::ToTokens;
    use schematic_core::container::Container;
    use syn::parse_quote;

    fn partial_type(input: syn::DeriveInput) -> String {
        let container = Container::from(input);
        let field = container.inner.get_fields()[0];

        field.value.get_partial_type().to_token_stream().to_string()
    }

    #[test]
    fn keeps_the_wrapper_of_unsized_types() {
        for ty in ["Path", "OsStr", "CStr"] {
            let ty: syn::Type = syn::parse_str(&format!("Box<{ty}>")).unwrap();

            assert_eq!(
                partial_type(parse_quote! {
                    #[derive(Config)]
                    struct Example {
                        a: #ty,
                    }
                })
                .replace(' ', ""),
                ty.to_token_stream().to_string().replace(' ', "")
            );
        }
    }

    #[test]
    fn keeps_the_box_of_a_recursive_config() {
        assert_eq!(
            partial_type(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    next: Option<Box<Self>>,
                }
            })
            .replace(' ', ""),
            "Option<Box<<ExampleasConfig>::Partial>>".replace("asConfig", "asschematic::Config")
        );
    }

    #[test]
    #[should_panic(expected = "must be wrapped in a `Box` or a collection")]
    fn errors_for_a_recursive_config_without_a_box() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                next: Option<Self>,
            }
        });
    }

    #[test]
    #[should_panic(expected = "must be optional, like `Option<Box<Self>>`")]
    fn errors_for_a_recursive_config_that_isnt_optional() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                next: Box<Self>,
            }
        });
    }

    // A generic type that isn't a known layer is the nested config itself
    #[test]
    fn treats_a_generic_as_the_nested_config() {
        assert_eq!(
            partial_type(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(nested)]
                    inner: Vec<Wrapper<String>>,
                }
            })
            .replace(' ', ""),
            "Vec<<Wrapper<String>asschematic::Config>::Partial>"
        );
    }

    // A wrapper the partial strips has to be rebuilt within it
    #[test]
    #[should_panic(expected = "is not a known collection")]
    fn errors_for_an_unknown_collection_of_wrappers() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                items: Custom<std::sync::Arc<String>>,
            }
        })
        .to_token_stream();
    }

    #[test]
    #[should_panic(expected = "Cannot use `context` without `nested`.")]
    fn errors_for_a_context_without_nested() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(context = schematic::context::unit)]
                value: String,
            }
        });
    }
}
