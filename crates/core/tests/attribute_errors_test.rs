mod utils;

use quote::ToTokens;
use schematic_core::container::{Container, ContainerMacro};
use starbase_sandbox::assert_snapshot;
use syn::{DeriveInput, parse_quote};
use utils::pretty;

// Attributes that parse but would do nothing panic, as a silent no-op is worse
// than a loud failure

fn config(input: DeriveInput) -> Container {
    let container = Container::from(input);
    container.validate_attributes();
    container
}

fn config_enum(input: DeriveInput) -> Container {
    let mut container = Container::from(input);
    container.macro_type = ContainerMacro::ConfigUnitEnum;
    container.validate_attributes();
    container
}

mod before_parse {
    use super::*;

    #[test]
    #[should_panic(expected = "Cannot use `before_parse` on a `Config`")]
    fn errors_on_a_config() {
        config(parse_quote! {
            #[derive(Config)]
            #[config(before_parse = "lowercase")]
            struct Example {
                a: String,
            }
        });
    }

    #[test]
    fn supports_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(before_parse = "lowercase")]
            enum Example {
                A,
            }
        });
    }
}

mod enums {
    use super::*;

    #[test]
    #[should_panic(expected = "Cannot use `fallback` on a `Config` enum variant")]
    fn errors_for_fallback_on_a_config() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                A,
                #[setting(fallback)]
                Other(String),
            }
        });
    }

    #[test]
    fn supports_fallback_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                A,
                #[variant(fallback)]
                Other(String),
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `#[default]` on a `Config` enum variant")]
    fn errors_for_std_default_on_a_config() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                #[default]
                A,
                B(String),
            }
        });
    }

    // A config enum doesn't implement `Default`, so the std derive can
    #[test]
    fn supports_std_default_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum, Default)]
            enum Example {
                #[default]
                A,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `validate` on a `ConfigEnum`")]
    fn errors_for_validate_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(validate = check)]
            enum Example {
                A,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `env_prefix` on a `Config` enum")]
    fn errors_for_env_prefix_on_a_config() {
        config(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "APP_")]
            enum Example {
                A(String),
            }
        });
    }

    // A named variant's settings derive keys with it, as a struct's do
    #[test]
    fn accepts_env_prefix_on_a_config_with_named_variants() {
        config(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "APP_")]
            enum Example {
                A(String),
                B { b: String },
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `env_prefix` on a `ConfigEnum` enum")]
    fn errors_for_env_prefix_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(env_prefix = "APP_")]
            enum Example {
                A,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Cannot use `allow_unknown_fields` on a `Config` enum")]
    fn errors_for_allow_unknown_fields_on_a_config() {
        config(parse_quote! {
            #[derive(Config)]
            #[config(allow_unknown_fields)]
            enum Example {
                A(String),
            }
        });
    }

    // A named variant has fields of its own
    #[test]
    fn supports_allow_unknown_fields_with_named_variants() {
        config(parse_quote! {
            #[derive(Config)]
            #[config(allow_unknown_fields)]
            enum Example {
                A(String),
                B { b: String },
            }
        });
    }

    #[test]
    #[should_panic(expected = "Can only use `rename_all` with named variants")]
    fn errors_for_rename_all_on_a_tuple_variant() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                #[serde(rename_all = "camelCase")]
                A(String),
            }
        });
    }

    #[test]
    #[should_panic(expected = "Only unit variants are supported")]
    fn errors_for_named_variants_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                A,
                B { b: String },
            }
        })
        .impl_config_enum();
    }

    #[test]
    #[should_panic(expected = "Cannot use `allow_unknown_fields` on a `ConfigEnum` enum")]
    fn errors_for_allow_unknown_fields_on_a_config_enum() {
        config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(allow_unknown_fields)]
            enum Example {
                A,
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Cannot use `tag` or `untagged` on a `Config` enum of only unit variants"
    )]
    fn errors_for_a_tag_on_unit_variants() {
        config(parse_quote! {
            #[derive(Config)]
            #[serde(tag = "type")]
            enum Example {
                A,
                B,
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Cannot use `tag` or `untagged` on a `Config` enum of only unit variants"
    )]
    fn errors_for_untagged_unit_variants() {
        config(parse_quote! {
            #[derive(Config)]
            #[serde(untagged)]
            enum Example {
                A,
                B,
            }
        });
    }

    #[test]
    fn supports_a_tag_on_variants_with_values() {
        config(parse_quote! {
            #[derive(Config)]
            #[serde(tag = "type")]
            enum Example {
                A,
                B(Child),
            }
        });
    }
}

mod env_prefix {
    use super::*;

    // A parent's prefix only applies to the keys a nested config derives with
    // an `env_prefix` of its own, which the nested config's partial reports
    #[test]
    fn asserts_a_nested_config_derives_keys() {
        let container = config(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested, env_prefix = "CHILD_")]
                child: Child,
                #[setting(nested)]
                other: Other,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_checks()));
    }

    // A tuple struct's setting is labeled by its position
    #[test]
    fn asserts_an_unnamed_setting_derives_keys() {
        let container = config(parse_quote! {
            #[derive(Config)]
            struct Example(#[setting(nested, env_prefix = "CHILD_")] Child);
        });

        assert_snapshot!(pretty(container.impl_partial_checks()));
    }

    // A named variant's settings are checked too, labeled by their variant
    #[test]
    fn asserts_a_variant_setting_derives_keys() {
        let container = config(parse_quote! {
            #[derive(Config)]
            enum Example {
                A {
                    #[setting(nested, env_prefix = "CHILD_")]
                    child: Child,
                    #[setting(nested)]
                    other: Other,
                },
            }
        });

        assert_snapshot!(pretty(container.impl_partial_checks()));
    }
}

// An attribute that fails to parse is an error spanned to where it was
// written, which the derive reports as a compile error, rather than a panic
mod parse_errors {
    use super::*;

    fn errors(input: DeriveInput) -> Vec<String> {
        Container::new(input)
            .expect_err("an error")
            .flatten()
            .into_iter()
            .map(|error| error.to_string())
            .collect()
    }

    #[test]
    fn suggests_a_misspelled_setting_attribute() {
        assert_eq!(
            errors(parse_quote! {
                #[derive(Config)]
                struct Example {
                    #[setting(defualt = 1)]
                    a: u8,
                }
            }),
            ["Unknown field: `defualt`. Did you mean `default`?"]
        );
    }

    #[test]
    fn reports_every_error_at_once() {
        assert_eq!(
            errors(parse_quote! {
                #[derive(Config)]
                enum Example {
                    #[setting(nestd)]
                    A(Child),
                    B {
                        #[setting(requird)]
                        b: String,
                        #[setting(env = 1)]
                        c: String,
                    },
                    C(#[setting(nested = 1)] Child),
                }
            }),
            [
                "Unknown field: `nestd`. Did you mean `nested`?",
                "Unknown field: `requird`. Did you mean `required`?",
                "Unexpected type `int` at env",
                "Unexpected type `int` at nested",
            ]
        );
    }

    #[test]
    fn reports_a_container_attribute() {
        assert_eq!(
            errors(parse_quote! {
                #[derive(Config)]
                #[config(env_prefx = "APP_")]
                struct Example {
                    a: String,
                }
            }),
            ["Unknown field: `env_prefx`. Did you mean `env_prefix`?"]
        );
    }
}

// A misused attribute still panics, but names the setting or variant, as
// the panic only points at the derive
mod panic_names {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Setting `port`: Cannot use `required` with `default`, as the setting always has a value."
    )]
    fn names_a_setting() {
        config(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(required, default = 1)]
                port: Option<u16>,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Setting `1`: Nested configurations must be a `Config` type")]
    fn names_a_setting_by_position() {
        config(parse_quote! {
            #[derive(Config)]
            struct Example(String, #[setting(nested)] String);
        });
    }

    #[test]
    #[should_panic(expected = "Setting `File.type`: Cannot use `extend` within an enum variant")]
    fn names_a_setting_of_a_variant() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                File {
                    #[setting(extend)]
                    r#type: String,
                },
            }
        });
    }

    #[test]
    #[should_panic(expected = "Variant `Unit`: Cannot use `merge` with unit variants.")]
    fn names_a_variant() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                #[setting(merge = func)]
                Unit,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Variant `A`: Nested configurations must be a `Config` type")]
    fn names_the_variant_of_a_value() {
        config(parse_quote! {
            #[derive(Config)]
            enum Example {
                A(#[setting(nested)] bool),
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Setting `items`: Unable to convert `Custom<std::sync::Arc<String>>`"
    )]
    fn names_a_setting_while_generating() {
        config(parse_quote! {
            #[derive(Config)]
            struct Example {
                items: Custom<std::sync::Arc<String>>,
            }
        })
        .to_token_stream();
    }
}
