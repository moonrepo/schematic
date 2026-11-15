mod utils;

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
}
