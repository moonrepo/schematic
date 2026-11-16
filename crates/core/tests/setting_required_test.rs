mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod setting_required {
    use super::*;

    mod named_struct {
        use super::*;

        #[test]
        fn renders_validate_check() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: Option<String>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: Option<String>,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at required")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required = 123)]
                    a: Option<String>,
                }
            });
        }

        #[test]
        fn renders_validate_check_for_non_optional() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: String,
                    #[setting(required)]
                    b: Vec<String>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        #[test]
        fn skips_default_value_for_non_optional() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    a: String,
                    b: String,
                    #[setting(required)]
                    c: Vec<String>,
                    #[setting(required, nested)]
                    d: Vec<NestedConfig>,
                }
            });

            assert_snapshot!(pretty(container.impl_partial_default_values()));
        }

        #[test]
        fn is_not_optional_with_container_serde_default() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                #[serde(default)]
                struct Example {
                    #[setting(required)]
                    a: String,
                    b: String,
                }
            });
            let fields = container.inner.get_fields();

            assert!(!fields[0].is_optional());
            assert!(fields[1].is_optional());
        }

        #[test]
        fn accepts_optional_nested() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required, nested)]
                    a: Option<NestedConfig>,
                }
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `required` with `default`, as the setting always has a value."
        )]
        fn errors_with_default() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required, default = "abc")]
                    a: String,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `required` with `default`, as the setting always has a value."
        )]
        fn errors_with_default_on_option() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required, default = "abc")]
                    a: Option<String>,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `required` with `default`, as the setting always has a value."
        )]
        fn errors_with_serde_default() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required)]
                    #[serde(default)]
                    a: String,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `required` with a non-optional `nested` setting, as it always exists."
        )]
        fn errors_with_bare_nested() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required, nested)]
                    a: NestedConfig,
                }
            });
        }

        #[test]
        #[should_panic(
            expected = "Cannot use `required` with a non-optional `nested` setting, as it always exists."
        )]
        fn errors_with_bare_nested_in_wrapper() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(required, nested)]
                    a: Box<NestedConfig>,
                }
            });
        }
    }

    mod unnamed_struct {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required)]
                    Option<String>,
                );
            });
            let field = container.inner.get_fields()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at required")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required = 123)]
                    Option<String>,
                );
            });
        }

        #[test]
        fn renders_validate_check_for_non_optional() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                struct Example(
                    #[setting(required)]
                    String,
                );
            });

            assert_snapshot!(pretty(container.impl_partial_validate()));
        }
    }

    mod named_enum {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(required)]
                        a: Option<String>,
                        #[setting(required)]
                        b: String,
                    },
                }
            });
            let variant = container.inner.get_variants()[0];

            assert!(variant.settings[0].args.required);
            assert!(variant.settings[1].args.required);
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at required")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(required = 123)]
                        a: Option<String>,
                    },
                }
            });
        }

        // Required settings are checked when finalizing, under the variant
        #[test]
        fn validates_settings() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(required)]
                        a: Option<String>,
                        #[setting(required)]
                        b: String,
                        c: bool,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_validate()));
        }

        // Left out of the defaults, so it stays empty until a layer provides it
        #[test]
        fn skips_defaults() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(default)]
                    A {
                        #[setting(required)]
                        a: String,
                        b: String,
                    },
                }
            });
            assert_snapshot!(pretty(container.impl_partial_default_values()));
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` on a named variant")]
        fn errors_on_the_variant() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required)]
                    A {
                        a: Option<String>,
                    },
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with `default`")]
        fn errors_with_default() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(required, default = "a")]
                        a: String,
                    },
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with a non-optional `nested` setting")]
        fn errors_for_bare_nested() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    A {
                        #[setting(required, nested)]
                        a: NestedConfig,
                    },
                }
            });
        }
    }

    mod unnamed_enum {
        use super::*;

        #[test]
        fn accepts_bool() {
            let container = Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required)]
                    A(Option<String>),
                }
            });
            let field = container.inner.get_variants()[0];

            assert!(field.args.required);
        }

        #[test]
        #[should_panic(expected = "Unexpected type `int` at required")]
        fn errors_invalid_type() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required = 123)]
                    A(Option<String>),
                }
            });
        }

        #[test]
        #[should_panic(expected = "Cannot use `required` with non-optional settings.")]
        fn errors_no_option() {
            Container::from(parse_quote! {
                enum Example {
                    #[setting(required)]
                    A(String),
                }
            });
        }
    }

    mod unit_enum {
        use super::*;

        #[test]
        #[should_panic(expected = "Cannot use `required` with unit variants.")]
        fn errors_for_unit() {
            Container::from(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(required)]
                    A,
                }
            });
        }
    }
}
